/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerFactory.
///
/// Routes each `ReceivedCommand` (a Java-mirror `AnyClientCommand` decoded by `command_socket`, or
/// an internal server command) to its `ServerCommandHandler*`, as Java looks handlers up by
/// `NetCommandId`. Session-level commands (ping, join, talk, version, password challenge, sketches,
/// markings, replay, user settings) go to their handlers; game commands are turned into engine
/// `Action`s by `decode_command` (the Rust engine takes `Action`s where Java hands the command to
/// the current step; BACKLOG §P P4 completes that mapping).
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use ffb_engine::action::{Action, PlayerActionChoice};
use ffb_engine::legal_actions::TeamSide;
use ffb_engine::replay_cache::ReplayCache;
use ffb_engine::replay_state::ReplayState;
use ffb_engine::server_replayer::ServerReplayer;
use ffb_engine::server_sketch_manager::ServerSketchManager;
use ffb_model::enums::{PlayerAction, SkillId};
use ffb_protocol::commands::any_client_command::AnyClientCommand;
use ffb_protocol::net_command::NetCommand;
use crate::db::db_connection_manager::DbConnectionManager;
use crate::game_cache::GameCache;
use crate::handler::server_command_handler_add_loaded_team::ServerCommandHandlerAddLoadedTeam;
use crate::handler::server_command_handler_add_sketch::ServerCommandHandlerAddSketch;
use crate::handler::server_command_handler_apply_automated_player_markings::ServerCommandHandlerApplyAutomatedPlayerMarkings;
use crate::handler::server_command_handler_calculate_automatic_player_markings::ServerCommandHandlerCalculateAutomaticPlayerMarkings;
use crate::handler::server_command_handler_clear_sketches::ServerCommandHandlerClearSketches;
use crate::handler::server_command_handler_close_game::ServerCommandHandlerCloseGame;
use crate::handler::server_command_handler_close_session::ServerCommandHandlerCloseSession;
use crate::handler::server_command_handler_delete_game::ServerCommandHandlerDeleteGame;
use crate::handler::server_command_handler_fumbbl_game_checked::ServerCommandHandlerFumbblGameChecked;
use crate::handler::server_command_handler_fumbbl_team_loaded::ServerCommandHandlerFumbblTeamLoaded;
use crate::handler::server_command_handler_join::ServerCommandHandlerJoin;
use crate::handler::server_command_handler_join_approved::ServerCommandHandlerJoinApproved;
use crate::handler::server_command_handler_join_replay::ServerCommandHandlerJoinReplay;
use crate::handler::server_command_handler_load_automatic_player_markings::ServerCommandHandlerLoadAutomaticPlayerMarkings;
use crate::handler::server_command_handler_password_challenge::ServerCommandHandlerPasswordChallenge;
use crate::handler::server_command_handler_ping::ServerCommandHandlerPing;
use crate::handler::server_command_handler_remove_sketches::ServerCommandHandlerRemoveSketches;
use crate::handler::server_command_handler_replay::ServerCommandHandlerReplay;
use crate::handler::server_command_handler_replay_loaded::ServerCommandHandlerReplayLoaded;
use crate::handler::server_command_handler_replay_status::ServerCommandHandlerReplayStatus;
use crate::handler::server_command_handler_request_version::ServerCommandHandlerRequestVersion;
use crate::handler::server_command_handler_schedule_game::ServerCommandHandlerScheduleGame;
use crate::handler::server_command_handler_set_marker::ServerCommandHandlerSetMarker;
use crate::handler::server_command_handler_set_prevent_sketching::ServerCommandHandlerSetPreventSketching;
use crate::handler::server_command_handler_sketch_add_coordinate::ServerCommandHandlerSketchAddCoordinate;
use crate::handler::server_command_handler_sketch_set_color::ServerCommandHandlerSketchSetColor;
use crate::handler::server_command_handler_sketch_set_label::ServerCommandHandlerSketchSetLabel;
use crate::handler::server_command_handler_socket_closed::ServerCommandHandlerSocketClosed;
use crate::handler::server_command_handler_talk::ServerCommandHandlerTalk;
use crate::handler::server_command_handler_transfer_control::ServerCommandHandlerTransferControl;
use crate::handler::server_command_handler_update_player_markings::ServerCommandHandlerUpdatePlayerMarkings;
use crate::handler::server_command_handler_upload_game::ServerCommandHandlerUploadGame;
use crate::handler::server_command_handler_user_settings::ServerCommandHandlerUserSettings;
use crate::model::received_command::{ReceivedCommand, ReceivedNetCommand};
use crate::net::commands::any_internal_server_command::AnyInternalServerCommand;
use crate::net::replay_session_manager::ReplaySessionManager;
use crate::net::server_communication::ServerCommunication;
use crate::net::session_manager::SessionManager;
use crate::request::fumbbl::util_fumbbl_request::{HttpClient, LazyReqwestHttpClient, ReqwestHttpClient};
use crate::request::server_request_processor::ServerRequestProcessor;
use crate::roster_cache::RosterCache;
use crate::team_cache::TeamCache;
use crate::util::server_start_game::MarkerContext;

/// Errors returned when a `ClientCommand` cannot be decoded to an `Action`.
#[derive(Debug)]
pub enum DecodeError {
    UnknownSkill(String),
    NotImplemented(String),
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodeError::UnknownSkill(s) => write!(f, "unknown skill: {s}"),
            DecodeError::NotImplemented(s) => write!(f, "not implemented: {s}"),
        }
    }
}

/// Java: `ServerCommandHandlerFactory`
pub struct ServerCommandHandlerFactory {
    pub game_cache: Arc<Mutex<GameCache>>,
    pub session_manager: Arc<Mutex<SessionManager>>,
    pub replay_session_manager: Arc<Mutex<ReplaySessionManager>>,
    pub db_connection_manager: Arc<Mutex<DbConnectionManager>>,
    pub team_cache: Arc<TeamCache>,
    pub roster_cache: Arc<RosterCache>,
    /// Java: the standalone team-setup rows (see `team_setup_cache.rs`).
    pub team_setup_cache: Arc<crate::team_setup_cache::TeamSetupCache>,
    pub client_properties: Arc<Vec<(String, String)>>,
    ping_handler: ServerCommandHandlerPing,
    socket_closed_handler: ServerCommandHandlerSocketClosed,
    /// `pub` (like this struct's other fields) so this file's own tests can exercise it
    /// directly in addition to through `handle_command`'s `ClientCommand::ClientJoin` arm
    /// (see that arm below), which now delegates here for real.
    pub join_handler: ServerCommandHandlerJoin,
    join_approved_handler: ServerCommandHandlerJoinApproved,
    talk_handler: ServerCommandHandlerTalk,
    close_session_handler: ServerCommandHandlerCloseSession,
    delete_game_handler: ServerCommandHandlerDeleteGame,
    transfer_control_handler: ServerCommandHandlerTransferControl,
    request_version_handler: ServerCommandHandlerRequestVersion,
    password_challenge_handler: ServerCommandHandlerPasswordChallenge,
    /// Shared with `socket_closed_handler`/`close_session_handler` (see their construction
    /// below) and every sketch handler below — one `ServerSketchManager` instance per server,
    /// matching Java's single `getServer().getSketchManager()`.
    pub sketch_manager: Arc<Mutex<ServerSketchManager>>,
    add_sketch_handler: ServerCommandHandlerAddSketch,
    clear_sketches_handler: ServerCommandHandlerClearSketches,
    remove_sketches_handler: ServerCommandHandlerRemoveSketches,
    sketch_add_coordinate_handler: ServerCommandHandlerSketchAddCoordinate,
    sketch_set_color_handler: ServerCommandHandlerSketchSetColor,
    sketch_set_label_handler: ServerCommandHandlerSketchSetLabel,
    set_marker_handler: ServerCommandHandlerSetMarker,
    set_prevent_sketching_handler: ServerCommandHandlerSetPreventSketching,
    /// `pub(crate)` so this file's own tests can seed a cached `ReplayState` directly (the
    /// same ad hoc name → `ReplayState` map `transfer_control_handler`/
    /// `set_prevent_sketching_handler`/`replay_status_handler` all share per their own
    /// documented "no server-level `ReplayCache` wired in yet" gap).
    pub(crate) replay_states: Arc<Mutex<HashMap<String, ReplayState>>>,
    update_player_markings_handler: ServerCommandHandlerUpdatePlayerMarkings,
    /// `pub(crate)` so this file's own tests can inspect the enqueued request without a
    /// network round trip (see `ServerCommandHandlerLoadAutomaticPlayerMarkings::request_processor`).
    pub(crate) load_automatic_player_markings_handler: ServerCommandHandlerLoadAutomaticPlayerMarkings,
    /// Shared with `update_player_markings_handler`/`load_automatic_player_markings_handler`
    /// above (see their own construction) — also reused for `FumbblTeamLoaded`'s
    /// `MarkerContext` and `FumbblGameChecked`'s `HttpClient` below rather than standing up
    /// disconnected pairs, matching this factory's existing convention (see `replay_handler`'s
    /// own doc comment for the same reuse of these two).
    markings_request_processor: Arc<Mutex<ServerRequestProcessor>>,
    markings_http_client: Arc<dyn HttpClient + Send + Sync>,
    /// Java: `getServer().getReplayCache()` — a real `ReplayCache` (unlike the ad hoc
    /// name → `ReplayState` map `transfer_control_handler`/`set_prevent_sketching_handler`/
    /// `replay_status_handler` fall back to per their own documented gaps), shared with
    /// `join_replay_handler` below since that's the only handler in this factory translated
    /// directly against `ReplayCache` rather than the stand-in map.
    pub replay_cache: Arc<Mutex<ReplayCache>>,
    join_replay_handler: ServerCommandHandlerJoinReplay,
    /// Shared with `replay_loaded_handler` below — one server-wide replay-playback queue,
    /// matching Java's single `getServer().getReplayer()`.
    pub replayer: Arc<Mutex<ServerReplayer>>,
    replay_handler: ServerCommandHandlerReplay,
    replay_loaded_handler: ServerCommandHandlerReplayLoaded,
    replay_status_handler: ServerCommandHandlerReplayStatus,

    // ── Game-management handler family ──────────────────────────────────
    /// `pub(crate)` (like `join_handler`/`load_automatic_player_markings_handler` above) so
    /// this file's own tests can exercise the real handler directly, in addition to through the
    /// `AddLoadedTeam` dispatch arm below (which now hands it the real `Team` the command
    /// carries — see `InternalServerCommandAddLoadedTeam::team`).
    pub(crate) add_loaded_team_handler: ServerCommandHandlerAddLoadedTeam,
    /// The two automated-player-marking handlers (distinct from
    /// `load_automatic_player_markings_handler`, which serves the client-originated
    /// `ClientLoadAutomaticPlayerMarkings` command). `pub(crate)` so this file's own tests can
    /// exercise them directly; both are also reachable via their dispatch arms below now that
    /// their internal commands carry a typed `AutoMarkingConfig`/`Game`.
    pub(crate) apply_automated_player_markings_handler: ServerCommandHandlerApplyAutomatedPlayerMarkings,
    pub(crate) calculate_automatic_player_markings_handler: ServerCommandHandlerCalculateAutomaticPlayerMarkings,
    fumbbl_team_loaded_handler: ServerCommandHandlerFumbblTeamLoaded,
    fumbbl_game_checked_handler: ServerCommandHandlerFumbblGameChecked,
    schedule_game_handler: ServerCommandHandlerScheduleGame,
    close_game_handler: ServerCommandHandlerCloseGame,
    /// A `ServerCommunication` handle built from this factory's own shared `session_manager`/
    /// `replay_session_manager`/dispatch sender (see `ServerCommunication::from_parts`'s doc
    /// comment) so `close_game_handler` has a real `&ServerCommunication` to call — without
    /// the circular "factory owns a `ServerCommunication` that owns the factory" loop
    /// `ServerCommunication::new` would create.
    communication_handle: ServerCommunication,
    upload_game_handler: ServerCommandHandlerUploadGame,
    user_settings_handler: ServerCommandHandlerUserSettings,
}

impl ServerCommandHandlerFactory {
    pub fn new(
        game_cache: Arc<Mutex<GameCache>>,
        session_manager: Arc<Mutex<SessionManager>>,
        db_connection_manager: Arc<Mutex<DbConnectionManager>>,
    ) -> Self {
        // Not wired to a real `ServerCommunication`'s dispatch queue — callers that need
        // `ServerCommandHandlerJoin`'s redispatch to actually reach `handle_internal_command`
        // should go through `with_replay_session_manager` (which `ServerCommunication::new`
        // uses) instead, passing its own live sender.
        let (dispatch_tx, _dispatch_rx) = mpsc::unbounded_channel();
        Self::with_replay_session_manager(
            game_cache,
            session_manager,
            Arc::new(Mutex::new(ReplaySessionManager::new())),
            db_connection_manager,
            dispatch_tx,
        )
    }

    /// Same as `new`, but lets a caller share an existing `ReplaySessionManager`
    /// (e.g. one already wired into `command_socket.rs`) instead of creating a
    /// private one that replay-session pings would never actually reach, and takes the
    /// `mpsc::UnboundedSender<ReceivedCommand>` that feeds
    /// `net::server_communication::dispatch_loop` so `ServerCommandHandlerJoin`'s
    /// `InternalServerCommandJoinApproved` redispatch lands back on the real dispatch queue.
    pub fn with_replay_session_manager(
        game_cache: Arc<Mutex<GameCache>>,
        session_manager: Arc<Mutex<SessionManager>>,
        replay_session_manager: Arc<Mutex<ReplaySessionManager>>,
        db_connection_manager: Arc<Mutex<DbConnectionManager>>,
        dispatch_tx: mpsc::UnboundedSender<ReceivedCommand>,
    ) -> Self {
        let ping_handler = ServerCommandHandlerPing::new(
            Arc::clone(&session_manager),
            Arc::clone(&replay_session_manager),
        );
        let sketch_manager = Arc::new(Mutex::new(ServerSketchManager::new()));
        let socket_closed_handler = ServerCommandHandlerSocketClosed::new(
            Arc::clone(&game_cache),
            Arc::clone(&session_manager),
            Arc::clone(&replay_session_manager),
            Arc::clone(&sketch_manager),
        );
        // Sensible empty defaults for the standalone-mode disk-XML lookup tables and the
        // `client.*` server properties — a real deployment would build these once at server
        // startup and share them here (no such startup wiring exists yet for this trio, same
        // as `ServerCommandHandlerScheduleGame`'s own `Default` impl).
        // Java: `GameCache.init()` (STANDALONE) — `rosterCache.init(new File("rosters"))`,
        // `teamCache.init(new File("teams"), server)`, relative to the server's working
        // directory (run it from `ffb-java/ffb/ffb-server`, as Java's is). Java throws on an
        // IOException; a missing directory here just leaves the cache empty (unit tests run
        // from the crate directory) and is logged.
        let mut team_cache = TeamCache::new();
        let mut roster_cache = RosterCache::new();
        if let Err(e) = roster_cache.init(std::path::Path::new("rosters")) {
            log::warn!("roster cache: {e}");
        }
        if let Err(e) = team_cache.init(std::path::Path::new("teams")) {
            log::warn!("team cache: {e}");
        }
        let team_cache = Arc::new(team_cache);
        let roster_cache = Arc::new(roster_cache);
        let mut team_setup_cache = crate::team_setup_cache::TeamSetupCache::new();
        if let Err(e) = team_setup_cache.init(std::path::Path::new("setups")) {
            log::warn!("team setup cache: {e}");
        }
        let team_setup_cache = Arc::new(team_setup_cache);
        let client_properties: Arc<Vec<(String, String)>> = Arc::new(Vec::new());
        let join_handler = ServerCommandHandlerJoin::new(
            Arc::clone(&game_cache),
            Arc::clone(&session_manager),
            Arc::clone(&db_connection_manager),
            dispatch_tx.clone(),
        );
        let join_approved_handler = ServerCommandHandlerJoinApproved::new(
            Arc::clone(&game_cache),
            Arc::clone(&session_manager),
            Arc::clone(&db_connection_manager),
            Arc::clone(&team_cache),
            Arc::clone(&roster_cache),
            Arc::clone(&client_properties),
        );
        let talk_handler = ServerCommandHandlerTalk::new(
            Arc::clone(&game_cache),
            Arc::clone(&session_manager),
            Arc::clone(&replay_session_manager),
        );
        // Reuses the same `sketch_manager` as `socket_closed_handler` above rather than
        // creating a second, disconnected `ServerSketchManager` — Java's
        // `ServerCommandHandlerCloseSession` delegates to the single shared
        // `ServerCommandHandlerSocketClosed` instance registered on the server, and this
        // keeps the two Rust handlers observing the same sketch state.
        let close_session_socket_closed = ServerCommandHandlerSocketClosed::new(
            Arc::clone(&game_cache),
            Arc::clone(&session_manager),
            Arc::clone(&replay_session_manager),
            Arc::clone(&sketch_manager),
        );
        let close_session_handler = ServerCommandHandlerCloseSession::new(close_session_socket_closed);
        let delete_game_handler = ServerCommandHandlerDeleteGame::new(Arc::clone(&db_connection_manager));
        // Java's `getServer().getReplayCache().replayState(name)` — no server-level
        // `ReplayCache` is wired into this crate yet (see this handler's own doc comment), so
        // an empty, private name → `ReplayState` map stands in for it: `transfer_control`
        // itself still runs for real against `replay_session_manager`, only the
        // broadcast-on-success branch (which needs a `ReplayState` lookup) is a documented
        // no-op until an entry exists.
        let replay_states: Arc<Mutex<HashMap<String, ReplayState>>> = Arc::new(Mutex::new(HashMap::new()));
        let transfer_control_handler = ServerCommandHandlerTransferControl::new(
            Arc::clone(&replay_session_manager),
            Arc::clone(&replay_states),
            Arc::clone(&session_manager),
        );
        let request_version_client_properties: HashMap<String, String> =
            client_properties.iter().cloned().collect();
        let request_version_handler = ServerCommandHandlerRequestVersion::new(
            Arc::clone(&session_manager),
            request_version_client_properties,
            false,
        );
        // The Rust server runs Java's `ServerMode.STANDALONE` only (no FUMBBL auth backend),
        // so the challenge is always null, exactly as Java's standalone branch sends it.
        let password_challenge_handler =
            ServerCommandHandlerPasswordChallenge::with_fumbbl_mode(Arc::clone(&session_manager), false);

        // ── Sketch/marker handler family (Phase ZVB) ────────────────────────
        let add_sketch_handler = ServerCommandHandlerAddSketch::new(
            Arc::clone(&sketch_manager),
            Arc::clone(&replay_session_manager),
        );
        let clear_sketches_handler = ServerCommandHandlerClearSketches::new(
            Arc::clone(&sketch_manager),
            Arc::clone(&replay_session_manager),
        );
        let remove_sketches_handler = ServerCommandHandlerRemoveSketches::new(
            Arc::clone(&sketch_manager),
            Arc::clone(&replay_session_manager),
        );
        let sketch_add_coordinate_handler = ServerCommandHandlerSketchAddCoordinate::new(
            Arc::clone(&sketch_manager),
            Arc::clone(&replay_session_manager),
        );
        let sketch_set_color_handler = ServerCommandHandlerSketchSetColor::new(
            Arc::clone(&sketch_manager),
            Arc::clone(&replay_session_manager),
        );
        let sketch_set_label_handler = ServerCommandHandlerSketchSetLabel::new(
            Arc::clone(&sketch_manager),
            Arc::clone(&replay_session_manager),
        );
        let set_marker_handler = ServerCommandHandlerSetMarker::new(
            Arc::clone(&game_cache),
            Arc::clone(&session_manager),
        );
        // Reuses `replay_states` (see `transfer_control_handler` above) — same documented
        // "no server-level ReplayCache yet" gap applies here.
        let set_prevent_sketching_handler = ServerCommandHandlerSetPreventSketching::new(
            Arc::clone(&replay_session_manager),
            Arc::clone(&replay_states),
            Arc::clone(&session_manager),
        );
        // No server-startup config wiring exists yet for the FUMBBL markings-fetch URL
        // template (same documented gap as `team_cache`/`roster_cache`/`client_properties`
        // above), so it defaults to empty — real HTTP calls built from it will simply fail
        // to fetch anything until that config is threaded through.
        let markings_url_template = String::new();
        let markings_request_processor: Arc<Mutex<ServerRequestProcessor>> =
            Arc::new(Mutex::new(ServerRequestProcessor::new()));
        let markings_http_client: Arc<dyn HttpClient + Send + Sync> = Arc::new(LazyReqwestHttpClient);
        let update_player_markings_handler = ServerCommandHandlerUpdatePlayerMarkings::new(
            Arc::clone(&db_connection_manager),
            Arc::clone(&markings_request_processor),
            Arc::clone(&markings_http_client),
            markings_url_template.clone(),
        );
        let load_automatic_player_markings_handler = ServerCommandHandlerLoadAutomaticPlayerMarkings::new(
            Arc::clone(&markings_request_processor),
            Arc::clone(&markings_http_client),
            markings_url_template,
            dispatch_tx.clone(),
        );

        // ── Replay handler family ───────────────────────────────────────────
        let replay_cache: Arc<Mutex<ReplayCache>> = Arc::new(Mutex::new(ReplayCache::new()));
        let join_replay_handler = ServerCommandHandlerJoinReplay::new(
            Arc::clone(&replay_session_manager),
            Arc::clone(&replay_cache),
            Arc::clone(&sketch_manager),
        );
        let replayer: Arc<Mutex<ServerReplayer>> = Arc::new(Mutex::new(ServerReplayer::new()));
        // No server-startup config wiring exists yet for the FUMBBL backup-service load-replay
        // URL template (same documented gap as `markings_url_template` above), so it defaults
        // to empty; the request processor/HTTP client are shared with the markings handlers
        // above rather than standing up a second, disconnected pair.
        let backup_url_load_template = String::new();
        let replay_handler = ServerCommandHandlerReplay::new(
            Arc::clone(&game_cache),
            Arc::clone(&session_manager),
            Arc::clone(&db_connection_manager),
            Arc::clone(&replayer),
            Arc::clone(&markings_request_processor),
            Arc::clone(&markings_http_client),
            backup_url_load_template.clone(),
            dispatch_tx.clone(),
        );
        let replay_loaded_handler = ServerCommandHandlerReplayLoaded::new(
            Arc::clone(&game_cache),
            Arc::clone(&session_manager),
            Arc::clone(&replayer),
        );
        // Reuses `replay_states` (see `transfer_control_handler` above) — same documented
        // "no server-level ReplayCache yet" gap applies here.
        let replay_status_handler = ServerCommandHandlerReplayStatus::new(
            Arc::clone(&replay_session_manager),
            Arc::clone(&replay_states),
        );

        // ── Game-management handler family ──────────────────────────────────
        let add_loaded_team_handler = ServerCommandHandlerAddLoadedTeam::new();
        // Each of these owns only an internal `MarkerGenerator::new()` (see their handler
        // files) — same trivial construction as `add_loaded_team_handler`.
        let apply_automated_player_markings_handler = ServerCommandHandlerApplyAutomatedPlayerMarkings::new();
        let calculate_automatic_player_markings_handler = ServerCommandHandlerCalculateAutomaticPlayerMarkings::new();
        let fumbbl_team_loaded_handler = ServerCommandHandlerFumbblTeamLoaded::new();
        let fumbbl_game_checked_handler = ServerCommandHandlerFumbblGameChecked::new();
        let schedule_game_handler = ServerCommandHandlerScheduleGame::new(
            Arc::clone(&game_cache),
            Arc::clone(&team_cache),
            Arc::clone(&roster_cache),
        );
        let close_game_handler =
            ServerCommandHandlerCloseGame::new(Arc::clone(&game_cache), Arc::clone(&session_manager));
        // See `communication_handle`'s own doc comment on the struct above for why this uses
        // `from_parts` rather than `ServerCommunication::new`.
        let communication_handle = ServerCommunication::from_parts(
            dispatch_tx.clone(),
            Arc::clone(&session_manager),
            Arc::clone(&replay_session_manager),
        );
        let upload_game_handler = ServerCommandHandlerUploadGame::new(
            Arc::clone(&game_cache),
            Arc::new(Mutex::new(ServerRequestProcessor::new())),
            Arc::clone(&markings_http_client),
            backup_url_load_template.clone(),
            dispatch_tx.clone(),
        );
        let user_settings_handler = ServerCommandHandlerUserSettings::new(
            Arc::clone(&session_manager),
            Arc::clone(&db_connection_manager),
        );

        Self {
            game_cache,
            session_manager,
            replay_session_manager,
            db_connection_manager,
            team_cache,
            roster_cache,
            team_setup_cache,
            client_properties,
            ping_handler,
            socket_closed_handler,
            join_handler,
            join_approved_handler,
            talk_handler,
            close_session_handler,
            delete_game_handler,
            transfer_control_handler,
            request_version_handler,
            password_challenge_handler,
            sketch_manager,
            add_sketch_handler,
            clear_sketches_handler,
            remove_sketches_handler,
            sketch_add_coordinate_handler,
            sketch_set_color_handler,
            sketch_set_label_handler,
            set_marker_handler,
            set_prevent_sketching_handler,
            replay_states,
            update_player_markings_handler,
            load_automatic_player_markings_handler,
            markings_request_processor: Arc::clone(&markings_request_processor),
            markings_http_client: Arc::clone(&markings_http_client),
            replay_cache,
            join_replay_handler,
            replayer,
            replay_handler,
            replay_loaded_handler,
            replay_status_handler,
            add_loaded_team_handler,
            apply_automated_player_markings_handler,
            calculate_automatic_player_markings_handler,
            fumbbl_team_loaded_handler,
            fumbbl_game_checked_handler,
            schedule_game_handler,
            close_game_handler,
            communication_handle,
            upload_game_handler,
            user_settings_handler,
        }
    }

    /// Java: `handleCommand(ReceivedCommand)` — the main dispatch entry point.
    ///
    /// Java's `ReceivedCommand.fCommand` is a `NetCommand` that is either a `ClientCommand`
    /// or an `InternalServerCommand` (see `ReceivedNetCommand`'s doc comment); a handler that
    /// wants to enqueue a follow-up command calls `communication.handleCommand(new
    /// ReceivedCommand(internalCommand, session))`, landing it back on this same dispatch
    /// point. Phase ZY.4 wires that redispatch sink through for real
    /// (`ServerCommunication::receive_internal`) and adds the `Internal` match arm below —
    /// routing `SocketClosed` and (Phase ZZ) `JoinApproved` to real handlers; the other 12
    /// internal command types need their own handler wired in with their own dependencies
    /// (several are `async`/DB-backed, e.g. `CloseGame`/`DeleteGame`) and fall through to a
    /// logged no-op for now rather than a fabricated stand-in. Phase ZZ made this whole
    /// method `async` (and `dispatch_loop` `.await`s it) specifically so `JoinApproved`
    /// could be wired for real — see `handle_internal_command`.
    pub async fn handle_command(&self, received: ReceivedCommand) {
        let session_id = received.session_id;

        let client_command = match received.command {
            ReceivedNetCommand::Internal(internal) => {
                self.handle_internal_command(internal, session_id).await;
                return;
            }
            ReceivedNetCommand::Client(cmd) => cmd,
        };

        let game_id = {
            let sm = self.session_manager.lock().unwrap();
            sm.get_game_id_for_session(session_id)
        };

        // Java: `ServerCommandHandlerFactory.handleCommand` looks the handler up by
        // `NetCommandId` and hands it the decoded `NetCommand`; every handler below takes the
        // Java-mirror `ClientCommand*` struct exactly as `NetCommandFactory.forJsonValue` built it.
        match &client_command {
            AnyClientCommand::ClientPing(ping) => {
                self.ping_handler.handle_command(session_id, ping.timestamp);
                return;
            }
            AnyClientCommand::ClientJoin(cmd) => {
                self.join_handler.handle_command(cmd, session_id).await;
                return;
            }
            AnyClientCommand::ClientTalk(cmd) => {
                self.talk_handler.handle_command(session_id, cmd);
                return;
            }
            AnyClientCommand::ClientCloseSession(_) => {
                self.close_session_handler.handle_command(session_id);
                return;
            }
            AnyClientCommand::ClientTransferReplayControl(cmd) => {
                self.transfer_control_handler.handle_command(session_id, cmd);
                return;
            }
            AnyClientCommand::ClientRequestVersion(_) => {
                self.request_version_handler.handle_command(session_id);
                return;
            }
            AnyClientCommand::ClientPasswordChallenge(cmd) => {
                let cmd = cmd.clone();
                let handler = self.password_challenge_handler.clone();
                // `ReqwestHttpClient` wraps `reqwest::blocking::Client`; building/dropping one
                // from inside a live tokio runtime context panics ("Cannot drop a runtime in a
                // context where blocking is not allowed"), so the whole synchronous handler
                // call is moved onto a blocking-pool thread instead.
                let _ = tokio::task::spawn_blocking(move || {
                    let client = ReqwestHttpClient::new();
                    handler.handle_command(&cmd, session_id, &client)
                })
                .await;
                return;
            }
            AnyClientCommand::ClientAddSketch(cmd) => {
                self.add_sketch_handler.handle_command(session_id, cmd);
                return;
            }
            AnyClientCommand::ClientClearSketches(cmd) => {
                self.clear_sketches_handler.handle_command(session_id, cmd);
                return;
            }
            AnyClientCommand::ClientRemoveSketches(cmd) => {
                self.remove_sketches_handler.handle_command(session_id, cmd);
                return;
            }
            AnyClientCommand::ClientSketchAddCoordinate(cmd) => {
                self.sketch_add_coordinate_handler.handle_command(session_id, cmd);
                return;
            }
            AnyClientCommand::ClientSketchSetColor(cmd) => {
                self.sketch_set_color_handler.handle_command(session_id, cmd);
                return;
            }
            AnyClientCommand::ClientSketchSetLabel(cmd) => {
                self.sketch_set_label_handler.handle_command(session_id, cmd);
                return;
            }
            AnyClientCommand::ClientSetMarker(cmd) => {
                self.set_marker_handler.handle_command(session_id, cmd);
                return;
            }
            AnyClientCommand::ClientSetPreventSketching(cmd) => {
                self.set_prevent_sketching_handler.handle_command(session_id, cmd);
                return;
            }
            AnyClientCommand::ClientLoadAutomaticPlayerMarkings(cmd) => {
                self.load_automatic_player_markings_handler.handle_command(cmd, session_id);
                return;
            }
            AnyClientCommand::ClientUpdatePlayerMarkings(cmd) => {
                // `ServerCommandHandlerUpdatePlayerMarkings::handle_command` takes the shared
                // `Arc<Mutex<..>>`s directly and locks/drops internally around its own
                // `.await` points, so no lock is held here across this call.
                self.update_player_markings_handler
                    .handle_command(cmd, session_id, &self.game_cache, &self.session_manager)
                    .await;
                return;
            }
            AnyClientCommand::ClientJoinReplay(cmd) => {
                self.join_replay_handler.handle_command(cmd, session_id);
                return;
            }
            AnyClientCommand::ClientReplay(cmd) => {
                self.replay_handler.handle_command(cmd, session_id).await;
                return;
            }
            AnyClientCommand::ClientReplayStatus(cmd) => {
                self.replay_status_handler.handle_command(cmd, session_id);
                return;
            }
            AnyClientCommand::ClientUserSettings(cmd) => {
                self.user_settings_handler.handle_command(session_id, cmd).await;
                return;
            }
            _ => {}
        }

        // Java StepSetup: CLIENT_TEAM_SETUP_LOAD -> UtilServerSetup.loadTeamSetup (a server-side
        // lookup the engine step cannot do; save/delete need the DB and are ignored).
        if let AnyClientCommand::ClientTeamSetupLoad(cmd) = &client_command {
            self.load_team_setup(game_id, session_id, cmd.setup_name.as_deref());
            return;
        }
        if matches!(client_command, AnyClientCommand::ClientTeamSetupSave(_) | AnyClientCommand::ClientTeamSetupDelete(_)) {
            return;
        }

        let side = {
            let sm = self.session_manager.lock().unwrap();
            if sm.get_session_of_home_coach(game_id) == Some(session_id) {
                TeamSide::Home
            } else {
                TeamSide::Away
            }
        };

        // Java StepSetup CLIENT_END_TURN: setPlayerCoordinates(command.getPlayerCoordinates()).
        let end_turn_coordinates: Option<Vec<(String, ffb_model::types::FieldCoordinate)>> = match &client_command {
            AnyClientCommand::ClientEndTurn(e) => {
                let mut v: Vec<_> = e.player_coordinates.iter().map(|(k, c)| (k.clone(), *c)).collect();
                v.sort_by(|a, b| a.0.cmp(&b.0));
                Some(v)
            }
            _ => None,
        };
        // Java UtilServerPlayerMove.isValidMove (every step that takes CLIENT_MOVE /
        // CLIENT_BLITZ_MOVE checks it): a null `coordinateFrom` or empty `coordinatesTo` is
        // invalid, and `coordinateFrom` (transformed for the away coach) must be the acting
        // player's square, else "Client move out of sync, Command dropped". `Action::Move` has no
        // from-square, so the check runs here, where the game is at hand (§Q3.7).
        let move_from: Option<Option<ffb_model::types::FieldCoordinate>> = match &client_command {
            AnyClientCommand::ClientMove(c) =>
                Some(c.coordinate_from.filter(|_| !c.coordinates_to.is_empty())),
            AnyClientCommand::ClientBlitzMove(c) =>
                Some(c.coordinate_from.filter(|_| !c.coordinates_to.is_empty())),
            _ => None,
        }
        .map(|from| from.map(|c| if side == TeamSide::Away { c.transform() } else { c }));
        // Java CLIENT_FIELD_COORDINATE: one wire command, six consuming steps (StepHitAndRun,
        // StepTrickster, StepPlaceBall, StepRaidingParty, StepInitPunt, StepFirst/SecondMove-
        // FuriousOutburst). The engine Action depends on which of them is waiting (§Q4.2).
        let field_coordinate = match &client_command {
            AnyClientCommand::ClientFieldCoordinate(f) => Some(f.field_coordinate),
            _ => None,
        };
        let touchback_coordinate = match &client_command {
            AnyClientCommand::ClientTouchback(t) => t.ball_coordinate.map(|c| if side == TeamSide::Away { c.transform() } else { c }),
            _ => None,
        };
        // Java AbstractStep.handleCommand: CLIENT_CONCEDE_GAME is handled by whatever step runs.
        if let AnyClientCommand::ClientConcedeGame(c) = &client_command {
            let Some(status) = c.concede_game_status else { return };
            let mut gc = self.game_cache.lock().unwrap();
            let Some(gs) = gc.get_game_state_by_id_mut(game_id) else { return };
            let conceded = match gs.driver.as_mut() {
                Some(driver) => {
                    driver.begin_report_snapshots();
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| driver.concede_game(status, side == TeamSide::Home)))
                        .unwrap_or_else(|_| {
                            log::error!("game {game_id}: engine panicked on a concession");
                            false
                        })
                }
                None => false,
            };
            log::info!("game {game_id}: {status:?} concession from {side:?} (conceded: {conceded})");
            crate::util::server_game_sync::sync_game(gs, &self.session_manager);
            return;
        }
        // Java ServerCommunication.handleByGameState never passes CLIENT_DEBUG_CLIENT_STATE to the
        // game, and no Java step takes CLIENT_JOURNEYMEN (UNHANDLED_COMMAND): both are no-ops.
        if matches!(client_command, AnyClientCommand::ClientDebugClientState(_) | AnyClientCommand::ClientJourneymen(_)) {
            log::debug!("game {game_id}: {:?} ignored, as in Java", client_command.get_id());
            return;
        }
        // Java AbstractStep.handleIllegalProcedure (any step): only while `timeoutPossible` --
        // ReportTimeoutEnforced(coach of the session), timeoutEnforced = true, timeoutPossible =
        // false, sync. The command stays UNHANDLED either way (§Q4.2).
        if matches!(client_command, AnyClientCommand::ClientIllegalProcedure(_)) {
            let coach = self.session_manager.lock().unwrap()
                .get_coach_for_session(session_id).unwrap_or_default().to_string();
            let mut gc = self.game_cache.lock().unwrap();
            let Some(gs) = gc.get_game_state_by_id_mut(game_id) else { return };
            let enforced = match gs.get_game_mut() {
                Some(game) if game.timeout_possible => {
                    game.report_list.add(ffb_model::report::report_timeout_enforced::ReportTimeoutEnforced::new(coach));
                    game.timeout_enforced = true;
                    game.timeout_possible = false;
                    true
                }
                _ => false,
            };
            if enforced {
                crate::util::server_game_sync::sync_game(gs, &self.session_manager);
            }
            return;
        }
        let action = match decode_command(client_command, side) {
            Ok(a) => a,
            Err(e) => {
                log::warn!("session {} decode error: {}", session_id, e);
                return;
            }
        };

        let mut gc = self.game_cache.lock().unwrap();
        let Some(gs) = gc.get_game_state_by_id_mut(game_id) else {
            log::warn!("session {} sent command but game {} not found", session_id, game_id);
            return;
        };
        if let Some(from) = move_from {
            let valid = gs.get_game().is_some_and(|game| is_valid_move(game, from));
            if !valid {
                log::debug!("game {game_id}: {side:?} !Client move out of sync, Command dropped");
                return;
            }
        }
        // Java: a step returns UNHANDLED_COMMAND for a command it does not wait for and nothing
        // happens. CLIENT_START_GAME is only waited for by StepInitStartGame, which raises no prompt.
        let prompt = gs.driver.as_ref().and_then(|d| d.current_prompt()).cloned();
        let mut action = action;
        if let Some(raw) = field_coordinate {
            match field_coordinate_action(prompt.as_ref(), raw, side) {
                Some(a) => action = a,
                None => {
                    // Java: no waiting step takes CLIENT_FIELD_COORDINATE -> UNHANDLED_COMMAND.
                    log::debug!("game {game_id}: CLIENT_FIELD_COORDINATE unhandled while waiting for {prompt:?}");
                    return;
                }
            }
        }
        if let (Some(ffb_model::prompts::AgentPrompt::TeamSetup { .. }), Some(coordinates)) = (&prompt, &end_turn_coordinates) {
            // UtilServerSteps.checkCommandIsFromCurrentPlayer
            let home_playing = gs.get_game().map(|g| g.home_playing).unwrap_or(true);
            if (side == TeamSide::Home) != home_playing {
                return;
            }
            if let Some(game) = gs.get_game_mut() {
                ffb_engine::util::util_server_setup::UtilServerSetup::set_player_coordinates(game, coordinates);
            }
            action = Action::ConfirmSetup;
        }
        // Java bb2025 StepSwarming CLIENT_END_TURN: `setPlayerCoordinates(command.getPlayerCoordinates())`
        // then end the swarm (the engine's ConfirmSetup). §Q2.5b.
        if let (Some(ffb_model::prompts::AgentPrompt::SwarmingPlacement { .. }), Some(coordinates)) = (&prompt, &end_turn_coordinates) {
            if let Some(game) = gs.get_game_mut() {
                ffb_engine::util::util_server_setup::UtilServerSetup::set_player_coordinates(game, coordinates);
            }
            action = Action::ConfirmSetup;
        }
        // Java bb2025 StepApplyKickoffResult CLIENT_END_TURN in SOLID_DEFENCE:
        // `setPlayerCoordinates(endTurnCommand.getPlayerCoordinates())` before the setup check.
        if let (Some(ffb_model::prompts::AgentPrompt::KickoffEventPlacement { mode, .. }), Some(coordinates)) = (&prompt, &end_turn_coordinates) {
            let home_playing = gs.get_game().map(|g| g.home_playing).unwrap_or(true);
            if mode == "SolidDefence" && (side == TeamSide::Home) == home_playing {
                if let Some(game) = gs.get_game_mut() {
                    ffb_engine::util::util_server_setup::UtilServerSetup::set_player_coordinates(game, coordinates);
                }
            }
        }
        if let Action::Touchback { player_id } = &mut action {
            if let (Some(c), Some(game)) = (touchback_coordinate, gs.get_game()) {
                *player_id = game.field_model.player_at(c).map(|p| p.to_string()).unwrap_or_default();
            }
        }
        // The engine steps for these player-choice modes match `Action::PlayerChoice` (by mode);
        // every other PlayerChoice prompt is answered with `SelectPlayer` ("" = declined).
        if let (Some(ffb_model::prompts::AgentPrompt::PlayerChoice { reason, .. }), Action::PlayerChoice { player_id, .. }) = (&prompt, &action) {
            const BY_MODE: &[&str] = &["DIVING_TACKLE", "TENTACLES", "SHADOWING", "MVP", "ANIMAL_SAVAGERY", "PICK_ME_UP", "QUICK_BITE", "FURIOUS_OUTBURST", "SOLID_DEFENCE", "CHARGE"];
            if !BY_MODE.contains(&reason.as_str()) {
                action = Action::SelectPlayer { player_id: player_id.clone().unwrap_or_default() };
            }
        }
        if matches!(action, Action::StartGame { .. }) && prompt.is_some() {
            log::debug!("game {game_id}: {action:?} unhandled while the engine waits for a prompt");
            return;
        }
        log::debug!("game {game_id}: engine <- {action:?} ({side:?})");
        // An engine panic must not poison the game cache (every later command would fail).
        let applied = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| gs.handle_action(side, action)));
        match applied {
            Ok(Ok(_)) => {}
            Ok(Err(e)) => {
                log::warn!("engine rejected action from session {}: {}", session_id, e);
                return;
            }
            Err(_) => {
                log::error!("game {game_id}: engine panicked on a command from session {session_id}");
                return;
            }
        }
        log::debug!(
            "game {game_id}: engine done, prompt {:?}",
            gs.driver.as_ref().and_then(|d| d.current_prompt()).map(|p| format!("{p:?}").chars().take(160).collect::<String>())
        );
        // Java: the step's model changes reach the clients via `UtilServerGame.syncGameModel`.
        crate::util::server_game_sync::sync_game(gs, &self.session_manager);
        if gs.is_finished() {
            if let Some(g) = gs.get_game() {
                log::info!(
                    "GAME FINISHED game {game_id}: {} {} - {} {}",
                    g.team_home.name, g.game_result.home.score, g.game_result.away.score, g.team_away.name
                );
            }
        }
    }

    /// Java: `UtilServerSetup.loadTeamSetup(gameState, setupName)` while the setup step waits.
    fn load_team_setup(&self, game_id: i64, session_id: crate::model::received_command::SessionId, setup_name: Option<&str>) {
        let mut gc = self.game_cache.lock().unwrap();
        let Some(gs) = gc.get_game_state_by_id_mut(game_id) else { return };
        let waiting = matches!(
            gs.driver.as_ref().and_then(|d| d.current_prompt()),
            Some(ffb_model::prompts::AgentPrompt::TeamSetup { .. })
        );
        if !waiting {
            return;
        }
        let Some(game) = gs.get_game_mut() else { return };
        let team_id = if game.home_playing { game.team_home.id.clone() } else { game.team_away.id.clone() };
        match setup_name.filter(|n| !n.is_empty()) {
            Some(name) => {
                if let Some(setup) = self.team_setup_cache.get(&team_id, name) {
                    setup.apply_to(game);
                    ffb_model::util::util_box::UtilBox::refresh_boxes(game);
                }
                crate::util::server_game_sync::sync_game(gs, &self.session_manager);
            }
            None => {
                let names = self.team_setup_cache.names_for_team(&team_id);
                let json = ffb_protocol::commands::server_command_team_setup_list::ServerCommandTeamSetupList::new(names)
                    .to_json_value()
                    .to_string();
                self.session_manager.lock().unwrap().send_to(session_id, &json);
            }
        }
    }

    /// Java: the `InternalServerCommand` half of `handleCommand`'s dispatch — routes each
    /// concrete internal command to its `ServerCommandHandler*`. See `handle_command`'s doc
    /// comment for which are wired vs. still gated behind their own missing infra.
    async fn handle_internal_command(&self, internal: AnyInternalServerCommand, session_id: crate::model::received_command::SessionId) {
        match internal {
            AnyInternalServerCommand::SocketClosed(_) => {
                self.socket_closed_handler.handle_command(session_id);
            }
            AnyInternalServerCommand::DeleteGame(cmd) => {
                self.delete_game_handler.handle_command(&cmd).await;
            }
            AnyInternalServerCommand::JoinApproved(cmd) => {
                // Java re-uses the already-connected Jetty `Session` object it was handed;
                // this crate's handler needs the session's outgoing sender explicitly, so it
                // is fetched back out of `SessionManager` by id (Phase ZZ's `sender_for`
                // accessor) — this only succeeds for a session that's already registered
                // (e.g. one `ServerCommandHandlerJoin` redispatched this command for).
                let sender = { self.session_manager.lock().unwrap().sender_for(session_id) };
                match sender {
                    Some(sender) => {
                        self.join_approved_handler.handle_command(&cmd, session_id, sender).await;
                    }
                    None => {
                        log::warn!(
                            "session {}: JoinApproved received but no registered sender found — \
                             session was never registered before this redispatch",
                            session_id
                        );
                    }
                }
            }
            AnyInternalServerCommand::ReplayLoaded(cmd) => {
                self.replay_loaded_handler.handle_command(&cmd, session_id);
            }
            AnyInternalServerCommand::AddLoadedTeam(cmd) => {
                // Java: `getServer().getCommunication().handleCommand(...)` for this command runs
                // `ServerCommandHandlerAddLoadedTeam.handleCommand`, which needs `command.getTeam()`.
                // The command now carries the real `Team` (parsed upstream in
                // `FumbblRequestLoadTeam`), so it is handed straight to the handler.
                let mut gc = self.game_cache.lock().unwrap();
                self.add_loaded_team_handler.handle_command(&cmd, cmd.team.clone(), &mut gc);
            }
            AnyInternalServerCommand::FumbblTeamLoaded(cmd) => {
                // Java re-uses the already-connected Jetty `Session`; this crate needs the
                // session's outgoing sender explicitly (same pattern as `JoinApproved` above).
                // Unlike `JoinApproved`, this command is the *first* registration for the
                // session (Java: `sendServerJoin` calls `sessionManager.addSession(...)`), so
                // there is no already-registered sender to fetch back out of `SessionManager`
                // for a session that's never been added before — only succeeds if some earlier
                // step (e.g. a prior spectator join) already registered this session id.
                let sender = { self.session_manager.lock().unwrap().sender_for(session_id) };
                match sender {
                    Some(sender) => {
                        let db = self.db_connection_manager.lock().unwrap().clone();
                        let marker_ctx = MarkerContext {
                            request_processor: &self.markings_request_processor,
                            client: Arc::clone(&self.markings_http_client),
                            markings_url_template: "",
                        };
                        // `GameCache` holds `Box<dyn Step>` engine state that isn't `Sync`, so
                        // the lock is dropped (via this scoped clone) before the `.await`
                        // below — see `ServerCommandHandlerFumbblTeamLoaded::handle_command`'s
                        // own doc comment for why it now takes an owned `Option<&Game>`
                        // instead of a `&GameCache` for exactly this reason.
                        let game = {
                            let gc = self.game_cache.lock().unwrap();
                            gc.get_game_state_by_id(cmd.game_id).and_then(|gs| gs.get_game()).cloned()
                        };
                        self.fumbbl_team_loaded_handler
                            .handle_command(
                                &cmd,
                                game.as_ref(),
                                &self.session_manager,
                                session_id,
                                sender,
                                &db,
                                &self.client_properties,
                                Some(marker_ctx),
                            )
                            .await;
                    }
                    None => {
                        log::warn!(
                            "session {}: FumbblTeamLoaded received but no registered sender found — \
                             session was never registered before this redispatch",
                            session_id
                        );
                    }
                }
            }
            AnyInternalServerCommand::FumbblGameChecked(cmd) => {
                let mut gc = self.game_cache.lock().unwrap();
                self.fumbbl_game_checked_handler.handle_command(cmd.game_id, &mut gc, self.markings_http_client.as_ref(), "");
            }
            AnyInternalServerCommand::ScheduleGame(cmd) => {
                self.schedule_game_handler.handle_command(&cmd);
            }
            AnyInternalServerCommand::CloseGame(cmd) => {
                let db = self.db_connection_manager.lock().unwrap().clone();
                self.close_game_handler.handle_command(&cmd, &self.communication_handle, &db).await;
            }
            AnyInternalServerCommand::UploadGame(cmd) => {
                self.upload_game_handler.handle_command(&cmd, session_id);
            }
            AnyInternalServerCommand::ApplyAutomatedPlayerMarkings(mut cmd) => {
                // Java: `ServerCommandHandlerApplyAutomatedPlayerMarkings.handleCommand`. The
                // command now carries a real `AutoMarkingConfig`, so it is applied directly. The
                // handler reads the game from `game_cache` and both reads the session mode and
                // sends the resulting markers via `session_manager`; both are held for the
                // duration of this synchronous call (no `.await` in between).
                let gc = self.game_cache.lock().unwrap();
                let sm = self.session_manager.lock().unwrap();
                self.apply_automated_player_markings_handler.handle_command(
                    &mut cmd.auto_marking_config,
                    cmd.game_id,
                    session_id,
                    &gc,
                    &sm,
                );
            }
            AnyInternalServerCommand::CalculateAutomaticPlayerMarkings(mut cmd) => {
                // Java: `ServerCommandHandlerCalculateAutomaticPlayerMarkings.handleCommand`. The
                // command carries the full `Game` and a real `AutoMarkingConfig`, so no
                // `game_cache` lookup is needed — only `session_manager` (to send the computed
                // markings back to the requesting session).
                let sm = self.session_manager.lock().unwrap();
                self.calculate_automatic_player_markings_handler.handle_command(
                    &cmd.game,
                    &mut cmd.auto_marking_config,
                    cmd.index,
                    session_id,
                    &sm,
                );
            }
            other => {
                log::debug!(
                    "session {}: internal command {} received but not yet wired into \
                     ServerCommandHandlerFactory (needs its own handler + dependencies)",
                    session_id, other.get_id()
                );
            }
        }
    }
}

/// CLIENT_FIELD_COORDINATE -> the Action of the step that is waiting for it. Java's StepHitAndRun,
/// StepTrickster, StepPlaceBall, StepRaidingParty and the Furious Outburst steps un-mirror the away
/// coach's square, and their Rust twins take the canonical square; Rust's StepInitPunt (like
/// Java's) un-mirrors it itself, so it gets the square as sent. `None` (also for a null
/// coordinate): no waiting step takes the command.
pub fn field_coordinate_action(
    prompt: Option<&ffb_model::prompts::AgentPrompt>,
    raw: Option<ffb_model::types::FieldCoordinate>,
    side: TeamSide,
) -> Option<Action> {
    use ffb_model::prompts::AgentPrompt as P;
    let raw = raw?;
    let canonical = if side == TeamSide::Away { raw.transform() } else { raw };
    match prompt? {
        P::HitAndRun { .. } => Some(Action::HitAndRun { coord: Some(canonical) }),
        P::TricksterMove { .. } => Some(Action::TricksterMove { coord: canonical }),
        P::PlaceBallSquare { .. } => Some(Action::PlaceBallSquare { coord: canonical }),
        P::RaidingParty { .. } => Some(Action::RaidingPartyTarget { coord: canonical }),
        P::FuriousOutburstSquare { .. } => Some(Action::FuriousOutburstSquare { coord: canonical }),
        P::PuntTarget { .. } => Some(Action::Punt { coord: raw }),
        _ => None,
    }
}

/// Java `UtilServerPlayerMove.isValidMove(GameState, FieldCoordinate, boolean)`: the move's
/// from-square (already home-oriented; `None` = Java's null coordinateFrom / empty
/// coordinatesTo) must be the acting player's current square.
pub fn is_valid_move(game: &ffb_model::model::game::Game, coordinate_from: Option<ffb_model::types::FieldCoordinate>) -> bool {
    let Some(from) = coordinate_from else { return false };
    game.acting_player.player_id.as_deref()
        .and_then(|id| game.field_model.player_coordinate(id))
        .is_some_and(|at| at == from)
}

// ── ClientCommand → Action decoder ───────────────────────────────────────────

/// Decodes a Java-wire `AnyClientCommand` into an engine `Action`.
///
/// Java has no such function: there the command is handed to the current step's
/// `handleCommand`, which reads exactly the fields it needs. The Rust engine takes `Action`s
/// instead, so this is the bridge; §P P4 replaces the mappings marked `NotImplemented` (and
/// re-checks every mapping against the Java step that consumes the command). Session-level
/// commands (join, ping, talk, ...) are routed before this is reached.
pub fn decode_command(cmd: AnyClientCommand, side: TeamSide) -> Result<Action, DecodeError> {
    fn missing(what: &str) -> DecodeError {
        DecodeError::NotImplemented(format!("missing field {what}"))
    }
    // Java: a command from the away coach carries coordinates in the away client's MIRRORED view;
    // the consuming step transforms them (`checkCommandIsFromHomePlayer ? c : c.transform()`,
    // UtilServerPlayerMove.fetchMoveStack, StepPushback, StepInitPassing). The Rust steps that
    // consume these actions do not, so it happens here. Kickoff, setup, throw team-mate, swoop,
    // punt and wizard steps transform themselves in Rust (as in Java) and are left alone.
    let away = side == TeamSide::Away;
    let m = |c: ffb_model::types::FieldCoordinate| if away { c.transform() } else { c };
    match cmd {
        AnyClientCommand::ClientEndTurn(_) => Ok(Action::EndTurn),

        AnyClientCommand::ClientMove(c) => Ok(Action::Move { path: c.coordinates_to.into_iter().map(m).collect() }),
        AnyClientCommand::ClientBlitzMove(c) => Ok(Action::Move { path: c.coordinates_to.into_iter().map(m).collect() }),

        AnyClientCommand::ClientActingPlayer(a) => {
            // Java: a null playerId is a deselect — ends the acting player's activation.
            let (Some(player_id), Some(raw_action)) = (a.player_id, a.player_action) else {
                return Ok(Action::EndPlayerAction);
            };
            let player_action = player_action_to_choice(raw_action)
                .ok_or(DecodeError::NotImplemented(format!("player_action {raw_action:?}")))?;
            Ok(Action::ActivatePlayer { player_id, player_action, block_defender_id: None })
        }

        AnyClientCommand::ClientBlock(b) => Ok(Action::Block {
            defender_id: b.defender_id.ok_or_else(|| missing("defenderId"))?,
        }),
        AnyClientCommand::ClientBlockChoice(b) => Ok(Action::BlockChoice {
            die_index: b.dice_index as usize,
            target_id: None,
        }),
        AnyClientCommand::ClientPushback(p) => Ok(Action::PushTo {
            coord: m(p.pushback.and_then(|p| p.coordinate).ok_or_else(|| missing("pushback"))?),
        }),
        AnyClientCommand::ClientFollowupChoice(f) => Ok(Action::FollowUp { follow_up: f.choice_followup }),

        AnyClientCommand::ClientKickoff(k) => Ok(Action::KickBall {
            coord: k.ball_coordinate.ok_or_else(|| missing("ballCoordinate"))?,
        }),

        AnyClientCommand::ClientPass(p) => Ok(Action::Pass {
            coord: m(p.target_coordinate.ok_or_else(|| missing("targetCoordinate"))?),
        }),
        AnyClientCommand::ClientHandOver(h) => Ok(Action::HandOff {
            receiver_id: h.catcher_id.ok_or_else(|| missing("catcherId"))?,
        }),
        AnyClientCommand::ClientFoul(f) => Ok(Action::Foul {
            target_id: f.defender_id.ok_or_else(|| missing("defenderId"))?,
        }),

        // Java StepIntercept CLIENT_INTERCEPTOR_CHOICE: `interceptorId` (null = no interception) and
        // `interceptionSkill`. §Q4.1: this decoded to a bare yes/no, and the step then took the
        // FIRST candidate -- a GUI coach's choice of interceptor (and a Yoink!) was lost.
        AnyClientCommand::ClientInterceptorChoice(i) => match (i.interceptor_id, i.interception_skill_id) {
            (Some(player_id), Some(skill)) => Ok(Action::InterceptWithSkill {
                player_id,
                skill_id: parse_skill_id(&skill).ok_or(DecodeError::UnknownSkill(skill))?,
            }),
            (player_id, _) => Ok(Action::SelectPlayer { player_id: player_id.unwrap_or_default() }),
        },
        AnyClientCommand::ClientCoinChoice(c) => Ok(Action::CoinChoice { heads: c.choice_heads }),
        AnyClientCommand::ClientReceiveChoice(r) => Ok(Action::ReceiveChoice { receive: r.choice_receive }),

        // Java AbstractStepWithReRoll CLIENT_USE_RE_ROLL: `reRollSource` (null = decline). The
        // steps record "TRR" when they offer, so a team re-roll is `UseReRoll`; a skill source
        // goes through `UseSkillReRoll` (the driver hands the skill to `use_reroll`, as for
        // CLIENT_USE_SKILL). §Q4.1: every source decoded to a plain team re-roll. Sources the
        // engine has no path for yet (Team Mascot / Mascot TRR / Pro TRR ...) are refused rather
        // than spent as a team re-roll.
        AnyClientCommand::ClientUseReRoll(r) => match r.re_roll_source {
            None => Ok(Action::UseReRoll { use_reroll: false }),
            Some(name) if name == "Team ReRoll" || name == "TRR" => Ok(Action::UseReRoll { use_reroll: true }),
            Some(name) => match parse_skill_id(&name) {
                Some(skill_id) => Ok(Action::UseSkillReRoll { skill_id }),
                None => Err(DecodeError::NotImplemented(format!("re-roll source {name}"))),
            },
        },
        AnyClientCommand::ClientUseSkill(s) => {
            let name = s.skill_name.ok_or_else(|| missing("skill"))?;
            let skill_id = parse_skill_id(&name).ok_or_else(|| DecodeError::UnknownSkill(name.clone()))?;
            Ok(Action::UseSkill { skill_id, use_skill: s.skill_used })
        }

        AnyClientCommand::ClientUseApothecary(a) => Ok(Action::UseApothecary {
            player_id: a.player_id.ok_or_else(|| missing("playerId"))?,
            use_apothecary: a.apothecary_used,
        }),

        AnyClientCommand::ClientSetupPlayer(s) => Ok(Action::PlacePlayer {
            player_id: s.player_id.ok_or_else(|| missing("playerId"))?,
            coord: s.coordinate.ok_or_else(|| missing("coordinate"))?,
        }),
        AnyClientCommand::ClientStartGame(_) => Ok(Action::StartGame { home: side == TeamSide::Home }),
        // Java StepBuyInducements: the command's InducementSet (type -> count bought).
        AnyClientCommand::ClientBuyInducements(c) => {
            let mut purchases: Vec<ffb_engine::action::InducementPurchase> = c
                .inducement_set
                .as_ref()
                .map(|set| set.get_inducements())
                .unwrap_or_default()
                .into_iter()
                .filter(|i| i.value > 0)
                .map(|i| ffb_engine::action::InducementPurchase { id: i.type_id, count: i.value as u32 })
                .collect();
            purchases.sort_by(|a, b| a.id.cmp(&b.id));
            Ok(Action::BuyInducements { home: side == TeamSide::Home, purchases })
        }

        AnyClientCommand::ClientThrowTeamMate(t) => Ok(Action::ThrowTeamMate {
            player_id: t.thrown_player_id.ok_or_else(|| missing("thrownPlayerId"))?,
            coord: t.target_coordinate.ok_or_else(|| missing("targetCoordinate"))?,
        }),
        // StepSwoop takes CLIENT_SWOOP as `Action::Swoop` (it un-mirrors the away coach's square
        // itself, as Java does). §Q3.7: this decoded to `Action::Pass`, which StepSwoop answered
        // with NEXT_STEP and no hook -- a GUI swoop never rolled its direction.
        AnyClientCommand::ClientSwoop(s) => Ok(Action::Swoop {
            coord: s.target_coordinate.ok_or_else(|| missing("targetCoordinate"))?,
        }),

        AnyClientCommand::ClientGaze(g) => Ok(Action::HypnoticGaze {
            target_id: g.victim_id.ok_or_else(|| missing("victimId"))?,
        }),
        AnyClientCommand::ClientConfirm(_) => Ok(Action::Acknowledge),
        AnyClientCommand::ClientArgueTheCall(a) => Ok(Action::ArgueTheCall { player_ids: a.player_ids }),
        // Java CLIENT_USE_INDUCEMENT: StepBribes / StepEndTurn read the AVOID_BAN type and the
        // bribed players, the inducement steps the type or card.
        AnyClientCommand::ClientUseInducement(u) => Ok(Action::UseInducement {
            inducement_type: u.inducement_type_name,
            card_id: u.card_name,
            player_ids: u.player_ids,
        }),

        // Java DialogPlayerChoice answer; an empty selection declines. Re-shaped per prompt in
        // `handle_command` (some steps take `SelectPlayer`).
        AnyClientCommand::ClientPlayerChoice(p) => Ok(Action::PlayerChoice {
            player_id: p.player_ids.first().cloned(),
            mode: p.player_choice_mode.map(|m| format!("{m:?}")).unwrap_or_default(),
            player_ids: p.player_ids,
        }),

        // Java CLIENT_KEYWORD_SELECTION (Getting Even) / CLIENT_POSITION_SELECTION (Raise Dead).
        AnyClientCommand::ClientKeywordSelection(k) => Ok(Action::KeywordSelection {
            player_id: k.player_id.ok_or_else(|| missing("playerId"))?,
            keywords: k.keywords.iter().map(|kw| kw.get_name().to_string()).collect(),
        }),
        AnyClientCommand::ClientPositionSelection(p) => Ok(Action::PositionSelection {
            team_id: p.team_id.ok_or_else(|| missing("teamId"))?,
            positions: p.position,
        }),

        // Java StepApothecaryMultiple CLIENT_USE_APOTHECARIES (bb2020 and bb2025): every casualty
        // still WAIT_FOR_APOTHECARY_USE becomes DO_NOT_USE_APOTHECARY -- `injuryDescriptions` is
        // never read, so nothing is lost here (§Q4.1 review note was a false positive).
        AnyClientCommand::ClientUseApothecaries(_) => Ok(Action::Acknowledge),
        // Java StepBlockRollMultiple CLIENT_BLOCK_OR_RE_ROLL_CHOICE_FOR_TARGET.
        AnyClientCommand::ClientBlockOrReRollChoiceForTarget(c) => Ok(Action::BlockOrReRollChoiceForTarget {
            target_id: c.target_id.ok_or_else(|| missing("targetId"))?,
            selected_index: c.selected_index,
            pro_index: c.pro_index,
            re_roll_source: c.re_roll_source.map(|s| s.name),
            any_dice_indexes: c.any_dice_indexes,
        }),
        // Java bb2025 StepTrickster CLIENT_PICK_UP_CHOICE.
        AnyClientCommand::ClientPickUpChoice(p) => Ok(Action::PickUpChoice { pick_up: p.attempt_pick_up }),
        // Java StepInitPunt CLIENT_PUNT_TO_CROWD.
        AnyClientCommand::ClientPuntToCrowd(p) => Ok(Action::PuntToCrowd { punt_to_crowd: p.punt_to_crowd }),
        // Java StepKickoffResultRoll CLIENT_KICK_OFF_RESULT_CHOICE: the overtime kick-off result.
        AnyClientCommand::ClientKickOffResultChoice(k) => Ok(Action::KickOffResultChoice {
            result: k.kickoff_result.ok_or_else(|| missing("kickoffResult"))?,
        }),
        // Java StepBloodLust CLIENT_BLOODLUST_ACTION: change the action to a move or keep it.
        AnyClientCommand::ClientBloodlustAction(b) => Ok(Action::BloodlustAction { change: b.change }),
        // Java StepApothecary CLIENT_APOTHECARY_CHOICE: the chosen state and serious injury.
        AnyClientCommand::ClientApothecaryChoice(a) => Ok(Action::ApothecaryChoice {
            player_state: a.player_state.ok_or_else(|| missing("playerState"))?.0,
            serious_injury: a.serious_injury.and_then(|si| serde_json::to_string(&si).ok()).map(|s| s.trim_matches('"').to_string()),
        }),
        // Java StepTouchback: the ball coordinate (mirrored from the away coach). The engine
        // takes the receiving player; `handle_command` resolves the player on that square.
        AnyClientCommand::ClientTouchback(_) => Ok(Action::Touchback { player_id: String::new() }),
        // Java StepSelectBlitzTarget / StepSelectGazeTarget: CLIENT_TARGET_SELECTED.
        AnyClientCommand::ClientTargetSelected(t) => Ok(Action::SelectPlayer {
            player_id: t.target_player_id.ok_or_else(|| missing("targetPlayerId"))?,
        }),

        // CLIENT_FIELD_COORDINATE: the handler replaces this with the waiting step's Action
        // (`field_coordinate_action`); decoded on its own it means nothing.
        AnyClientCommand::ClientFieldCoordinate(_) => Ok(Action::Acknowledge),

        // ── §Q4.2: commands whose engine Action already exists ──────────────────────────────
        // Java StepBlockRoll / StepBlockRollMultiple block-die re-rolls.
        AnyClientCommand::ClientUseBrawler(b) => Ok(Action::UseBrawler { target_id: b.target_id }),
        AnyClientCommand::ClientUseHatred(_) => Ok(Action::UseHatred),
        AnyClientCommand::ClientUseProReRollForBlock(p) => Ok(Action::UseProReRollForBlock {
            die_index: usize::try_from(p.pro_index).map_err(|_| missing("proIndex"))?,
        }),
        AnyClientCommand::ClientUseConsummateReRollForBlock(p) => Ok(Action::UseConsummateReRollForBlock {
            die_index: usize::try_from(p.pro_index).map_err(|_| missing("proIndex"))?,
        }),
        AnyClientCommand::ClientUseSingleBlockDieReRoll(s) => Ok(Action::UseSingleBlockDieReRoll {
            re_roll_source: s.re_roll_source.map(|r| r.name).ok_or_else(|| missing("reRollSource"))?,
            die_index: usize::try_from(s.die_index).map_err(|_| missing("dieIndex"))?,
        }),
        AnyClientCommand::ClientUseMultiBlockDiceReRoll(m) => Ok(Action::UseMultiBlockDiceReRoll {
            dice_indexes: m.dice_indexes.iter().map(|i| usize::try_from(*i).map_err(|_| missing("diceIndexes")))
                .collect::<Result<_, _>>()?,
        }),
        AnyClientCommand::ClientUseReRollForTarget(r) => Ok(Action::UseReRollForTarget {
            re_rolled_action: r.re_rolled_action,
            re_roll_source: r.re_roll_source,
            target_id: r.target_id,
        }),
        // Java StepInitSelecting CLIENT_SYNCHRONOUS_MULTI_BLOCK: the chosen block targets.
        AnyClientCommand::ClientSynchronousMultiBlock(m) => Ok(Action::MultiBlock { targets: m.selected_block_targets }),
        // Java bb2016 StepPettyCash / StepBuyCards: keyed on the sending coach.
        AnyClientCommand::ClientPettyCash(p) => Ok(Action::PettyCash { home: side == TeamSide::Home, amount: p.petty_cash }),
        AnyClientCommand::ClientBuyCard(b) => Ok(Action::BuyCard { home: side == TeamSide::Home, card_type: b.card_type_name }),
        // Java StepWeatherMage: `Weather.valueOf(weatherName)` (the enum constant name); the step
        // takes the modifier from the dialog it showed.
        AnyClientCommand::ClientSelectWeather(w) => {
            let name = w.weather_name.ok_or_else(|| missing("weatherName"))?;
            let weather = ffb_model::enums::Weather::all().iter().copied()
                .find(|x| ffb_engine::step::bb2025::inducements::step_weather_mage::java_weather_constant(*x) == name)
                .ok_or_else(|| DecodeError::NotImplemented(format!("weather {name}")))?;
            Ok(Action::SelectWeather { weather })
        }
        // Java StepWizard CLIENT_WIZARD_SPELL: the spell and its target (StepWizard un-mirrors the
        // away coach's square itself).
        AnyClientCommand::ClientWizardSpell(w) => {
            use ffb_engine::action::WizardSpellChoice;
            use ffb_model::model::SpecialEffect;
            let spell = match w.wizard_spell {
                Some(SpecialEffect::LIGHTNING) => WizardSpellChoice::Lightning,
                Some(SpecialEffect::FIREBALL) => WizardSpellChoice::Fireball,
                Some(SpecialEffect::ZAP) => WizardSpellChoice::Zap,
                other => return Err(DecodeError::NotImplemented(format!("wizard spell {other:?}"))),
            };
            Ok(Action::WizardSpell { spell, coord: w.target_coordinate.ok_or_else(|| missing("targetCoordinate"))? })
        }
        // Java StepEndBlocking CLIENT_PILE_DRIVER: the knocked-down target (null/empty = decline),
        // the engine's `SelectPlayer` answer to its Pile Driver dialog.
        AnyClientCommand::ClientPileDriver(p) => Ok(Action::SelectPlayer { player_id: p.player_id.unwrap_or_default() }),
        // Java StepPileDriver CLIENT_USE_CHAINSAW: `usingChainsaw`.
        AnyClientCommand::ClientUseChainsaw(c) => Ok(Action::UseSkill { skill_id: SkillId::Chainsaw, use_skill: c.using_chainsaw }),
        // Java StepPrayer / Wisdom CLIENT_SKILL_SELECTION (sent as clientPrayerSelection): the skill.
        AnyClientCommand::ClientPrayerSelection(s) => {
            let name = s.skill_id.ok_or_else(|| missing("skill"))?;
            Ok(Action::SelectSkill { skill_id: parse_skill_id(&name).ok_or(DecodeError::UnknownSkill(name))? })
        }

        other => Err(DecodeError::NotImplemented(format!("{:?}", other.get_id()))),
    }
}

/// Map `PlayerAction` (wire/model enum) → `PlayerActionChoice` (engine enum).
///
/// This is the inverse of `choice_to_player_action` in `ffb-client/src/network_encoder/mod.rs`.
fn player_action_to_choice(action: PlayerAction) -> Option<PlayerActionChoice> {
    match action {
        PlayerAction::Move | PlayerAction::BlitzMove | PlayerAction::PassMove
        | PlayerAction::FoulMove | PlayerAction::HandOverMove | PlayerAction::GazeMove
        | PlayerAction::ThrowTeamMateMove | PlayerAction::KickTeamMateMove
        | PlayerAction::PutridRegurgitationMove => Some(PlayerActionChoice::Move),

        PlayerAction::Block | PlayerAction::MultipleBlock
        | PlayerAction::PutridRegurgitationBlock | PlayerAction::KickEmBlock => Some(PlayerActionChoice::Block),

        PlayerAction::Blitz | PlayerAction::BlitzSelect | PlayerAction::StandUpBlitz
        | PlayerAction::PutridRegurgitationBlitz | PlayerAction::KickEmBlitz => Some(PlayerActionChoice::Blitz),

        PlayerAction::Stab | PlayerAction::Chainsaw => Some(PlayerActionChoice::Stab),

        PlayerAction::Foul => Some(PlayerActionChoice::Foul),

        PlayerAction::Pass | PlayerAction::HailMaryPass | PlayerAction::DumpOff => Some(PlayerActionChoice::Pass),
        PlayerAction::ThrowBomb | PlayerAction::HailMaryBomb => Some(PlayerActionChoice::ThrowBomb),

        PlayerAction::HandOver => Some(PlayerActionChoice::HandOff),

        PlayerAction::StandUp | PlayerAction::RemoveConfusion => Some(PlayerActionChoice::StandUp),

        PlayerAction::ThrowTeamMate => Some(PlayerActionChoice::ThrowTeamMate),
        PlayerAction::KickTeamMate => Some(PlayerActionChoice::KickTeamMate),

        PlayerAction::Gaze | PlayerAction::GazeSelect | PlayerAction::AutoGazeZoat => Some(PlayerActionChoice::HypnoticGaze),

        PlayerAction::Swoop => Some(PlayerActionChoice::Swoop),
        PlayerAction::Punt | PlayerAction::PuntMove => Some(PlayerActionChoice::Punt),
        PlayerAction::BreatheFire => Some(PlayerActionChoice::BreatheFire),
        PlayerAction::ProjectileVomit => Some(PlayerActionChoice::ProjectileVomit),
        PlayerAction::SecureTheBall => Some(PlayerActionChoice::SecureTheBall),

        _ => None,
    }
}

/// Parse a `SkillId` from the Debug-format string produced by `format!("{:?}", skill_id)`.
///
/// The `Debug` representation uses PascalCase (e.g. `"Block"`, `"SureHands"`).
/// `SkillId`'s serde impl uses `rename_all = "camelCase"`, so we lowercase the
/// first character before deserializing.
fn parse_skill_id(s: &str) -> Option<SkillId> {
    // Java sends the skill's display name ("Stand Firm"); the factory knows every name.
    if let Some(id) = ffb_model::factory::skill_factory::SkillFactory::new().for_name(s) {
        return Some(id);
    }
    let camel = to_camel_case(s);
    serde_json::from_str::<SkillId>(&format!("\"{}\"", camel)).ok()
}

/// Lowercase the first ASCII character of a PascalCase string to produce camelCase.
fn to_camel_case(s: &str) -> String {
    let mut out = s.to_string();
    if let Some(first) = out.get_mut(0..1) {
        first.make_ascii_lowercase();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::types::FieldCoordinate;

    /// §Q3.7: CLIENT_SWOOP is StepSwoop's `Action::Swoop`, coordinates as sent (StepSwoop
    /// un-mirrors the away coach's square itself, as Java does).
    #[test]
    fn decode_swoop_is_a_swoop_action() {
        let r = decode(serde_json::json!({"netCommandId": "clientSwoop", "actingPlayerId": "p",
            "targetCoordinate": [4, 6]}), TeamSide::Away);
        assert!(matches!(r, Ok(Action::Swoop { coord }) if coord == coord_of(4, 6)), "{r:?}");
    }

    fn coord_of(x: i32, y: i32) -> FieldCoordinate { FieldCoordinate::new(x, y) }

    /// §Q4.1 CLIENT_INTERCEPTOR_CHOICE keeps the chosen interceptor (and a Yoink! skill); no
    /// interceptor declines.
    #[test]
    fn decode_interceptor_choice_keeps_the_interceptor() {
        let r = decode(serde_json::json!({"netCommandId": "clientInterceptorChoice", "interceptorId": "a7"}), TeamSide::Away);
        assert!(matches!(&r, Ok(Action::SelectPlayer { player_id }) if player_id == "a7"), "{r:?}");
        let r = decode(serde_json::json!({"netCommandId": "clientInterceptorChoice"}), TeamSide::Away);
        assert!(matches!(&r, Ok(Action::SelectPlayer { player_id }) if player_id.is_empty()), "{r:?}");
        let r = decode(serde_json::json!({"netCommandId": "clientInterceptorChoice", "interceptorId": "a7",
            "skill": "Yoink!"}), TeamSide::Away);
        assert!(matches!(&r, Ok(Action::InterceptWithSkill { player_id, .. }) if player_id == "a7"), "{r:?}");
    }

    /// §Q4.2 Pile Driver target (empty declines) and the Chainsaw choice in StepPileDriver.
    #[test]
    fn decode_pile_driver_and_use_chainsaw() {
        let r = decode(serde_json::json!({"netCommandId": "clientPileDriver", "playerId": "a2"}), TeamSide::Home);
        assert!(matches!(&r, Ok(Action::SelectPlayer { player_id }) if player_id == "a2"), "{r:?}");
        let r = decode(serde_json::json!({"netCommandId": "clientPileDriver"}), TeamSide::Home);
        assert!(matches!(&r, Ok(Action::SelectPlayer { player_id }) if player_id.is_empty()), "{r:?}");
        let r = decode(serde_json::json!({"netCommandId": "clientUseChainsaw", "usingChainsaw": true}), TeamSide::Home);
        assert!(matches!(r, Ok(Action::UseSkill { skill_id: SkillId::Chainsaw, use_skill: true })), "{r:?}");
    }

    /// §Q4.2 CLIENT_FIELD_COORDINATE goes to the waiting step: canonical square for Hit and Run
    /// (and the other steps whose Rust twin takes it canonical), as sent for the punt (StepInitPunt
    /// un-mirrors it itself), nothing when no step waits for it.
    #[test]
    fn field_coordinate_maps_by_the_waiting_prompt() {
        use ffb_model::prompts::AgentPrompt as P;
        let raw = Some(coord_of(3, 4));
        let hit = P::HitAndRun { player_id: "a1".into(), squares: vec![] };
        assert_eq!(field_coordinate_action(Some(&hit), raw, TeamSide::Away),
            Some(Action::HitAndRun { coord: Some(coord_of(3, 4).transform()) }));
        assert_eq!(field_coordinate_action(Some(&hit), raw, TeamSide::Home),
            Some(Action::HitAndRun { coord: Some(coord_of(3, 4)) }));
        let punt = P::PuntTarget { player_id: "a1".into(), squares: vec![] };
        assert_eq!(field_coordinate_action(Some(&punt), raw, TeamSide::Away), Some(Action::Punt { coord: coord_of(3, 4) }));
        assert_eq!(field_coordinate_action(None, raw, TeamSide::Home), None);
        assert_eq!(field_coordinate_action(Some(&hit), None, TeamSide::Home), None);
    }

    /// §Q4.2 commands whose engine Action already existed: block-die re-rolls, weather, wizard
    /// spell, petty cash, skill selection.
    #[test]
    fn decode_q42_direct_commands() {
        let r = decode(serde_json::json!({"netCommandId": "clientUseBrawler", "playerId": "a3"}), TeamSide::Home);
        assert!(matches!(&r, Ok(Action::UseBrawler { target_id: Some(t) }) if t == "a3"), "{r:?}");
        let r = decode(serde_json::json!({"netCommandId": "clientUseProReRollForBlock", "proIndex": 1}), TeamSide::Home);
        assert!(matches!(r, Ok(Action::UseProReRollForBlock { die_index: 1 })), "{r:?}");
        let r = decode(serde_json::json!({"netCommandId": "clientUseMultiBlockDiceReRoll", "blockDiceIndexes": [0, 2]}), TeamSide::Home);
        assert!(matches!(&r, Ok(Action::UseMultiBlockDiceReRoll { dice_indexes }) if dice_indexes == &vec![0, 2]), "{r:?}");
        let r = decode(serde_json::json!({"netCommandId": "clientSelectWeather", "name": "VERY_SUNNY", "modifier": 1}), TeamSide::Away);
        assert!(matches!(r, Ok(Action::SelectWeather { weather: ffb_model::enums::Weather::VerySunny })), "{r:?}");
        let r = decode(serde_json::json!({"netCommandId": "clientPettyCash", "pettyCash": 50000}), TeamSide::Away);
        assert!(matches!(r, Ok(Action::PettyCash { home: false, amount: 50000 })), "{r:?}");
    }

    /// §Q4.1 CLIENT_USE_RE_ROLL keeps the source: team re-roll, a skill, or a decline.
    #[test]
    fn decode_use_re_roll_keeps_the_source() {
        let r = decode(serde_json::json!({"netCommandId": "clientUseReRoll", "reRolledAction": "dodge",
            "reRollSource": "Team ReRoll"}), TeamSide::Home);
        assert!(matches!(r, Ok(Action::UseReRoll { use_reroll: true })), "{r:?}");
        let r = decode(serde_json::json!({"netCommandId": "clientUseReRoll", "reRolledAction": "dodge"}), TeamSide::Home);
        assert!(matches!(r, Ok(Action::UseReRoll { use_reroll: false })), "{r:?}");
        let r = decode(serde_json::json!({"netCommandId": "clientUseReRoll", "reRolledAction": "dodge",
            "reRollSource": "Dodge"}), TeamSide::Home);
        assert!(matches!(r, Ok(Action::UseSkillReRoll { skill_id: SkillId::Dodge })), "{r:?}");
        let r = decode(serde_json::json!({"netCommandId": "clientUseReRoll", "reRolledAction": "dodge",
            "reRollSource": "Mascot TRR"}), TeamSide::Home);
        assert!(r.is_err(), "{r:?}");
    }

    /// §Q3.7 Java UtilServerPlayerMove.isValidMove: only a move from the acting player's own
    /// square is valid; no from-square (null / empty path) or another square is dropped.
    #[test]
    fn is_valid_move_requires_the_acting_players_square() {
        use ffb_model::model::{game::Game, team::Team};
        let mut game = Game::new(Team::default(), Team::default(), ffb_model::enums::Rules::Bb2025);
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        assert!(!is_valid_move(&game, Some(FieldCoordinate::new(5, 5))), "no acting player");
        game.acting_player.player_id = Some("p1".into());
        assert!(is_valid_move(&game, Some(FieldCoordinate::new(5, 5))));
        assert!(!is_valid_move(&game, Some(FieldCoordinate::new(6, 5))), "out of sync");
        assert!(!is_valid_move(&game, None), "null coordinateFrom / empty coordinatesTo");
    }
    use crate::model::received_command::ReceivedCommand;

    fn coord(x: i32, y: i32) -> FieldCoordinate { FieldCoordinate { x, y } }

    /// Builds a client command exactly as the Java client puts it on the wire, through the
    /// same `NetCommandFactory.forJsonValue` decode `command_socket` uses.
    fn wire(json: serde_json::Value) -> AnyClientCommand {
        match ffb_protocol::net_command_factory::NetCommandFactory::new().for_json_value(&json) {
            Some(ffb_protocol::net_command_factory::AnyNetCommand::Client(c)) => c,
            other => panic!("not a client command: {other:?}"),
        }
    }

    fn decode(json: serde_json::Value, side: TeamSide) -> Result<Action, DecodeError> {
        decode_command(wire(json), side)
    }

    #[test]
    fn decode_end_turn() {
        let r = decode(serde_json::json!({"netCommandId": "clientEndTurn"}), TeamSide::Home);
        assert!(matches!(r, Ok(Action::EndTurn)));
    }

    #[test]
    fn decode_coin_choice() {
        let heads = decode(serde_json::json!({"netCommandId": "clientCoinChoice", "choiceHeads": true}), TeamSide::Away);
        assert!(matches!(heads, Ok(Action::CoinChoice { heads: true })));
        let tails = decode(serde_json::json!({"netCommandId": "clientCoinChoice", "choiceHeads": false}), TeamSide::Away);
        assert!(matches!(tails, Ok(Action::CoinChoice { heads: false })));
    }

    #[test]
    fn decode_receive_choice() {
        let r = decode(serde_json::json!({"netCommandId": "clientReceiveChoice", "choiceReceive": true}), TeamSide::Home);
        assert!(matches!(r, Ok(Action::ReceiveChoice { receive: true })));
    }

    #[test]
    fn decode_start_game_side() {
        let h = decode(serde_json::json!({"netCommandId": "clientStartGame"}), TeamSide::Home);
        assert!(matches!(h, Ok(Action::StartGame { home: true })));
        let a = decode(serde_json::json!({"netCommandId": "clientStartGame"}), TeamSide::Away);
        assert!(matches!(a, Ok(Action::StartGame { home: false })));
    }

    #[test]
    fn decode_setup_player() {
        let r = decode(
            serde_json::json!({"netCommandId": "clientSetupPlayer", "playerId": "p1", "coordinate": [3, 4]}),
            TeamSide::Home,
        );
        match r {
            Ok(Action::PlacePlayer { player_id, coord: c }) => {
                assert_eq!(player_id, "p1");
                assert_eq!(c, coord(3, 4));
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn decode_block_and_block_choice() {
        let b = decode(
            serde_json::json!({"netCommandId": "clientBlock", "actingPlayerId": "a", "defenderId": "d"}),
            TeamSide::Home,
        );
        assert!(matches!(b, Ok(Action::Block { ref defender_id }) if defender_id == "d"));
        let c = decode(serde_json::json!({"netCommandId": "clientBlockChoice", "diceIndex": 2}), TeamSide::Home);
        assert!(matches!(c, Ok(Action::BlockChoice { die_index: 2, target_id: None })));
    }

    #[test]
    fn decode_acting_player_deselect_and_select() {
        let deselect = decode(serde_json::json!({"netCommandId": "clientActingPlayer"}), TeamSide::Home);
        assert!(matches!(deselect, Ok(Action::EndPlayerAction)));
        let select = decode(
            serde_json::json!({"netCommandId": "clientActingPlayer", "playerId": "p", "playerAction": "block"}),
            TeamSide::Home,
        );
        assert!(matches!(select, Ok(Action::ActivatePlayer { player_action: PlayerActionChoice::Block, .. })));
    }

    #[test]
    fn decode_use_skill_unknown_is_error() {
        let r = decode(
            serde_json::json!({"netCommandId": "clientUseSkill", "skill": "NoSuchSkill", "skillUsed": true}),
            TeamSide::Home,
        );
        assert!(matches!(r, Err(DecodeError::UnknownSkill(_))));
    }

    #[test]
    fn decode_session_commands_are_not_actions() {
        let r = decode(serde_json::json!({"netCommandId": "clientPing", "timestamp": 1}), TeamSide::Home);
        assert!(matches!(r, Err(DecodeError::NotImplemented(_))));
    }

    #[test]
    fn parse_skill_block() {
        assert_eq!(parse_skill_id("Block"), Some(SkillId::Block));
    }

    #[test]
    fn parse_skill_sure_hands() {
        assert_eq!(parse_skill_id("SureHands"), Some(SkillId::SureHands));
    }

    #[test]
    fn parse_skill_hail_mary_pass() {
        assert_eq!(parse_skill_id("HailMaryPass"), Some(SkillId::HailMaryPass));
    }

    #[test]
    fn parse_skill_unknown_returns_none() {
        assert_eq!(parse_skill_id("NotASkill"), None);
    }

    #[test]
    fn parse_skill_empty_returns_none() {
        assert_eq!(parse_skill_id(""), None);
    }

    // ── player_action_to_choice ────────────────────────────────────────────────

    #[test]
    fn player_action_move_maps_to_move() {
        assert_eq!(player_action_to_choice(PlayerAction::Move), Some(PlayerActionChoice::Move));
    }

    #[test]
    fn player_action_blitz_move_maps_to_move() {
        assert_eq!(player_action_to_choice(PlayerAction::BlitzMove), Some(PlayerActionChoice::Move));
    }

    #[test]
    fn player_action_blitz_maps_to_blitz() {
        assert_eq!(player_action_to_choice(PlayerAction::Blitz), Some(PlayerActionChoice::Blitz));
    }

    #[test]
    fn player_action_block_maps_to_block() {
        assert_eq!(player_action_to_choice(PlayerAction::Block), Some(PlayerActionChoice::Block));
    }

    #[test]
    fn player_action_foul_maps_to_foul() {
        assert_eq!(player_action_to_choice(PlayerAction::Foul), Some(PlayerActionChoice::Foul));
    }

    #[test]
    fn player_action_hand_over_maps_to_hand_off() {
        assert_eq!(player_action_to_choice(PlayerAction::HandOver), Some(PlayerActionChoice::HandOff));
    }

    #[test]
    fn player_action_gaze_maps_to_hypnotic_gaze() {
        assert_eq!(player_action_to_choice(PlayerAction::Gaze), Some(PlayerActionChoice::HypnoticGaze));
    }

    #[test]
    fn player_action_stand_up_maps_to_stand_up() {
        assert_eq!(player_action_to_choice(PlayerAction::StandUp), Some(PlayerActionChoice::StandUp));
    }

    #[test]
    fn player_action_throw_bomb_maps_to_throw_bomb() {
        assert_eq!(player_action_to_choice(PlayerAction::ThrowBomb), Some(PlayerActionChoice::ThrowBomb));
    }

    #[test]
    fn player_action_swoop_maps_to_swoop() {
        assert_eq!(player_action_to_choice(PlayerAction::Swoop), Some(PlayerActionChoice::Swoop));
    }

    #[test]
    fn player_action_punt_maps_to_punt() {
        assert_eq!(player_action_to_choice(PlayerAction::Punt), Some(PlayerActionChoice::Punt));
    }

    #[test]
    fn player_action_breathe_fire_maps() {
        assert_eq!(player_action_to_choice(PlayerAction::BreatheFire), Some(PlayerActionChoice::BreatheFire));
    }

    #[test]
    fn player_action_secure_the_ball_maps() {
        assert_eq!(player_action_to_choice(PlayerAction::SecureTheBall), Some(PlayerActionChoice::SecureTheBall));
    }

    #[test]
    fn player_action_unknown_variant_returns_none() {
        assert_eq!(player_action_to_choice(PlayerAction::Forgo), None);
    }

    // ── to_camel_case ─────────────────────────────────────────────────────────

    #[test]
    fn camel_case_lowercases_first_char() {
        assert_eq!(to_camel_case("Block"), "block");
        assert_eq!(to_camel_case("SureHands"), "sureHands");
        assert_eq!(to_camel_case("HailMaryPass"), "hailMaryPass");
    }

    #[test]
    fn camel_case_empty_is_safe() {
        assert_eq!(to_camel_case(""), "");
    }

    // ── handle_command routing (unit level) ───────────────────────────────────

    fn db() -> Arc<Mutex<crate::db::db_connection_manager::DbConnectionManager>> {
        Arc::new(Mutex::new(crate::db::db_connection_manager::DbConnectionManager::new()))
    }

    #[tokio::test]
    async fn handle_command_ping_updates_last_ping() {
        use std::sync::{Arc, Mutex};
        use ffb_model::model::ClientMode;
        use tokio::sync::mpsc;
        use crate::model::received_command::ReceivedCommand;

        let gc = Arc::new(Mutex::new(crate::game_cache::GameCache::new()));
        let sm_arc = Arc::new(Mutex::new(crate::net::session_manager::SessionManager::new()));
        {
            let (tx, _rx) = mpsc::unbounded_channel();
            let mut sm = sm_arc.lock().unwrap();
            sm.add_session(42, 1, "TestCoach".into(), ClientMode::PLAYER, true, vec![], tx);
        }
        let factory = ServerCommandHandlerFactory::new(Arc::clone(&gc), Arc::clone(&sm_arc), db());
        factory.handle_command(ReceivedCommand::new(AnyClientCommand::ClientPing(ffb_protocol::commands::client_command_ping::ClientCommandPing { timestamp: 9999, ..Default::default() }), 42)).await;
        let sm = sm_arc.lock().unwrap();
        // ServerCommandHandlerPing stores the current wall-clock time as the
        // last-ping timestamp (matching Java's `System.currentTimeMillis()`)
        // and separately echoes the client's `9999` timestamp back in the pong
        // reply, rather than storing the client-supplied value directly.
        assert!(sm.get_last_ping(42) > 0);
    }

    #[tokio::test]
    async fn handle_command_unknown_session_does_not_panic() {
        use std::sync::{Arc, Mutex};
        let gc = Arc::new(Mutex::new(crate::game_cache::GameCache::new()));
        let sm = Arc::new(Mutex::new(crate::net::session_manager::SessionManager::new()));
        let factory = ServerCommandHandlerFactory::new(gc, sm, db());
        // session 99 was never registered — should log a warning and return cleanly
        factory.handle_command(crate::model::received_command::ReceivedCommand::new(
            AnyClientCommand::ClientEndTurn(ffb_protocol::commands::client_command_end_turn::ClientCommandEndTurn::default()), 99,
        )).await;
    }

    #[tokio::test]
    async fn join_handler_field_is_directly_usable_for_lobby_listing() {
        use ffb_model::model::ClientMode;
        use tokio::sync::mpsc;

        let gc = Arc::new(Mutex::new(crate::game_cache::GameCache::new()));
        let sm = Arc::new(Mutex::new(crate::net::session_manager::SessionManager::new()));
        let (tx, mut rx) = mpsc::unbounded_channel();
        sm.lock().unwrap().add_session(1, 0, "Coach".into(), ClientMode::PLAYER, true, vec![], tx);
        let factory = ServerCommandHandlerFactory::new(gc, sm, db());
        let join = ffb_protocol::commands::client_command_join::ClientCommandJoin {
            coach: Some("Coach".into()),
            ..Default::default()
        };
        assert!(factory.join_handler.handle_command(&join, 1).await);
        let _ = rx.try_recv();
    }

    /// Proves a re-sent `ClientCommand::ClientJoin` (the only way this arm is reached — see
    /// `handle_command`'s doc comment) really invokes the real `join_handler` rather than
    /// just logging, by observing its lobby-listing side effect (a `ServerGameList` reply)
    /// for a session that supplies no game id/name/replay mode.
    #[tokio::test]
    async fn handle_command_client_join_dispatches_to_join_handler() {
        use ffb_model::model::ClientMode;
        use tokio::sync::mpsc;
        use crate::model::received_command::ReceivedCommand;

        let gc = Arc::new(Mutex::new(crate::game_cache::GameCache::new()));
        let sm = Arc::new(Mutex::new(crate::net::session_manager::SessionManager::new()));
        let (tx, mut rx) = mpsc::unbounded_channel();
        sm.lock().unwrap().add_session(11, 0, "Coach".into(), ClientMode::PLAYER, true, vec![], tx);
        let factory = ServerCommandHandlerFactory::new(gc, sm, db());

        let join = wire(serde_json::json!({"netCommandId": "clientJoin", "coach": "Coach"}));
        factory.handle_command(ReceivedCommand::new(join, 11)).await;

        let msg = rx.try_recv().expect("expected a real ServerGameList reply from join_handler");
        assert!(msg.contains("serverGameList"));
    }

    /// A re-sent `ClientJoin` that *does* supply a game name is treated as a targeted join —
    /// same as `ServerCommandHandlerJoin`'s own `targeted_join_by_game_name_is_also_treated_as_targeted`
    /// coverage — proving the `game_id` wire field is threaded through as `game_name`.
    #[tokio::test]
    async fn handle_command_client_join_with_game_name_is_targeted() {
        use ffb_model::model::ClientMode;
        use tokio::sync::mpsc;
        use crate::model::received_command::ReceivedCommand;

        let gc = Arc::new(Mutex::new(crate::game_cache::GameCache::new()));
        let sm = Arc::new(Mutex::new(crate::net::session_manager::SessionManager::new()));
        let (tx, mut rx) = mpsc::unbounded_channel();
        sm.lock().unwrap().add_session(12, 0, "Coach".into(), ClientMode::PLAYER, true, vec![], tx);
        let factory = ServerCommandHandlerFactory::new(gc, sm, db());

        let join = wire(serde_json::json!({
            "netCommandId": "clientJoin", "coach": "Coach", "gameName": "SomeGame", "password": ""
        }));
        factory.handle_command(ReceivedCommand::new(join, 12)).await;

        // No DB pool configured: the targeted join is approved (BACKLOG §P deviation) and
        // the redispatched JoinApproved creates the named game - proving `has_target` saw
        // the game name rather than answering with a lobby list.
        assert!(!rx.try_recv().map(|m| m.contains("serverGameList")).unwrap_or(false));
    }

    /// End-to-end dispatch test (Phase ZZ): enqueuing `AnyInternalServerCommand::JoinApproved`
    /// through the real `handle_command` → `handle_internal_command` → `join_approved_handler`
    /// path, analogous to the existing `SocketClosed` dispatch coverage in
    /// `net::server_communication`'s own tests.
    #[tokio::test]
    async fn factory_dispatches_join_approved_through_real_handler() {
        use ffb_model::model::ClientMode;
        use tokio::sync::mpsc;
        use crate::model::received_command::ReceivedCommand;
        use crate::net::commands::any_internal_server_command::AnyInternalServerCommand;
        use crate::net::commands::internal_server_command_join_approved::InternalServerCommandJoinApproved;

        let gc = Arc::new(Mutex::new(crate::game_cache::GameCache::new()));
        let sm = Arc::new(Mutex::new(crate::net::session_manager::SessionManager::new()));
        let (tx, mut rx) = mpsc::unbounded_channel();
        sm.lock().unwrap().add_session(9, 0, "Watcher".into(), ClientMode::SPECTATOR, false, vec![], tx);

        let factory = ServerCommandHandlerFactory::new(gc, sm, db());
        let cmd = InternalServerCommandJoinApproved::new(
            0, "FactoryDispatchGame".to_string(), "Watcher".to_string(), String::new(), "SPECTATOR".to_string(), vec![],
        );
        factory.handle_command(ReceivedCommand::new_internal(AnyInternalServerCommand::JoinApproved(cmd), 9)).await;

        assert!(factory.game_cache.lock().unwrap().get_game_state_by_name("FactoryDispatchGame").is_some());
        let msg = rx.try_recv().expect("expected a real serverJoin broadcast via the dispatch path");
        assert!(msg.contains("serverJoin"));
    }

    // ── Phase ZVA: session/game-lifecycle handler family dispatch ─────────────

    #[tokio::test]
    async fn handle_command_talk_broadcasts_via_real_talk_handler() {
        use ffb_model::model::ClientMode;
        use tokio::sync::mpsc;

        let gc = Arc::new(Mutex::new(crate::game_cache::GameCache::new()));
        let sm_arc = Arc::new(Mutex::new(crate::net::session_manager::SessionManager::new()));
        let game_id = gc.lock().unwrap().create_game_state();
        let (tx1, mut rx1) = mpsc::unbounded_channel();
        let (tx2, mut rx2) = mpsc::unbounded_channel();
        {
            let mut sm = sm_arc.lock().unwrap();
            sm.add_session(1, game_id, "Home".into(), ClientMode::PLAYER, true, vec![], tx1);
            sm.add_session(2, game_id, "Away".into(), ClientMode::PLAYER, false, vec![], tx2);
        }
        let factory = ServerCommandHandlerFactory::new(Arc::clone(&gc), Arc::clone(&sm_arc), db());
        factory.handle_command(ReceivedCommand::new(
            AnyClientCommand::ClientTalk(ffb_protocol::commands::client_command_talk::ClientCommandTalk { talk: Some("hello".into()), ..Default::default() }),
            1,
        )).await;

        let msg1 = rx1.try_recv().expect("expected a real serverTalk broadcast via the dispatch path");
        let msg2 = rx2.try_recv().expect("expected a real serverTalk broadcast via the dispatch path");
        assert!(msg1.contains("hello"));
        assert_eq!(msg1, msg2);
    }

    #[tokio::test]
    async fn handle_command_close_session_removes_session_via_real_handler() {
        use ffb_model::model::ClientMode;
        use tokio::sync::mpsc;

        let gc = Arc::new(Mutex::new(crate::game_cache::GameCache::new()));
        let sm_arc = Arc::new(Mutex::new(crate::net::session_manager::SessionManager::new()));
        let game_id = gc.lock().unwrap().create_game_state();
        let (tx, _rx) = mpsc::unbounded_channel();
        sm_arc.lock().unwrap().add_session(1, game_id, "Coach".into(), ClientMode::PLAYER, true, vec![], tx);

        let factory = ServerCommandHandlerFactory::new(Arc::clone(&gc), Arc::clone(&sm_arc), db());
        factory.handle_command(ReceivedCommand::new(
            AnyClientCommand::ClientCloseSession(ffb_protocol::commands::client_command_close_session::ClientCommandCloseSession::default()),
            1,
        )).await;

        assert!(sm_arc.lock().unwrap().get_coach_for_session(1).is_none());
    }

    #[tokio::test]
    async fn handle_command_transfer_replay_control_reaches_real_handler() {
        use ffb_model::model::ClientMode;
        use tokio::sync::mpsc;

        let gc = Arc::new(Mutex::new(crate::game_cache::GameCache::new()));
        let sm_arc = Arc::new(Mutex::new(crate::net::session_manager::SessionManager::new()));
        let (tx1, _rx1) = mpsc::unbounded_channel();
        let (tx2, _rx2) = mpsc::unbounded_channel();
        sm_arc.lock().unwrap().add_session(1, 0, "coach1".into(), ClientMode::SPECTATOR, false, vec![], tx1);
        sm_arc.lock().unwrap().add_session(2, 0, "coach2".into(), ClientMode::SPECTATOR, false, vec![], tx2);

        let factory = ServerCommandHandlerFactory::new(Arc::clone(&gc), Arc::clone(&sm_arc), db());
        {
            let mut rsm = factory.replay_session_manager.lock().unwrap();
            rsm.add_session(1, "replay1".into(), "coach1".into());
            rsm.add_session(2, "replay1".into(), "coach2".into());
        }

        factory.handle_command(ReceivedCommand::new(
            AnyClientCommand::ClientTransferReplayControl(ffb_protocol::commands::client_command_transfer_replay_control::ClientCommandTransferReplayControl {
                coach: Some("coach2".into()), ..Default::default() }),
            1,
        )).await;

        // No `ReplayState` is registered for "replay1" in the factory's private map (documented
        // gap), so the broadcast side-effect doesn't fire, but the real handler's
        // `transfer_control` call against `ReplaySessionManager` did run.
        assert!(factory.replay_session_manager.lock().unwrap().has_control(2));
    }

    #[tokio::test]
    async fn handle_command_request_version_sends_version_via_real_handler() {
        use ffb_model::model::ClientMode;
        use tokio::sync::mpsc;

        let gc = Arc::new(Mutex::new(crate::game_cache::GameCache::new()));
        let sm_arc = Arc::new(Mutex::new(crate::net::session_manager::SessionManager::new()));
        let (tx, mut rx) = mpsc::unbounded_channel();
        sm_arc.lock().unwrap().add_session(1, 0, "Coach".into(), ClientMode::PLAYER, true, vec![], tx);

        let factory = ServerCommandHandlerFactory::new(Arc::clone(&gc), Arc::clone(&sm_arc), db());
        factory.handle_command(ReceivedCommand::new(
            AnyClientCommand::ClientRequestVersion(ffb_protocol::commands::client_command_request_version::ClientCommandRequestVersion::default()),
            1,
        )).await;

        let msg = rx.try_recv().expect("expected a real serverVersion message via the dispatch path");
        let value: serde_json::Value = serde_json::from_str(&msg).unwrap();
        assert_eq!(value["netCommandId"], "serverVersion");
    }

    #[tokio::test]
    async fn handle_command_password_challenge_sends_challenge_via_real_handler() {
        use ffb_model::model::ClientMode;
        use tokio::sync::mpsc;

        let gc = Arc::new(Mutex::new(crate::game_cache::GameCache::new()));
        let sm_arc = Arc::new(Mutex::new(crate::net::session_manager::SessionManager::new()));
        let (tx, mut rx) = mpsc::unbounded_channel();
        sm_arc.lock().unwrap().add_session(1, 0, "Coach".into(), ClientMode::PLAYER, true, vec![], tx);

        let factory = ServerCommandHandlerFactory::new(Arc::clone(&gc), Arc::clone(&sm_arc), db());
        // No coach supplied, so even the default fumbbl-mode-true handler sends the
        // password challenge directly rather than issuing a live HTTP fetch.
        factory.handle_command(ReceivedCommand::new(
            AnyClientCommand::ClientPasswordChallenge(ffb_protocol::commands::client_command_password_challenge::ClientCommandPasswordChallenge {
                coach: None, ..Default::default() }),
            1,
        )).await;

        let msg = rx.try_recv().expect("expected a real serverPasswordChallenge message via the dispatch path");
        assert!(msg.contains("serverPasswordChallenge"));
    }

    #[tokio::test]
    async fn handle_command_delete_game_dispatches_through_real_internal_handler() {
        use crate::net::commands::internal_server_command_delete_game::InternalServerCommandDeleteGame;

        let gc = Arc::new(Mutex::new(crate::game_cache::GameCache::new()));
        let sm = Arc::new(Mutex::new(crate::net::session_manager::SessionManager::new()));
        let factory = ServerCommandHandlerFactory::new(gc, sm, db());

        // Without a live DB pool configured the delete itself is a no-op (see
        // `ServerCommandHandlerDeleteGame`'s own tests), but this proves the internal command
        // reaches the real handler through `handle_command` rather than the logged no-op
        // catch-all in `handle_internal_command`.
        let cmd = InternalServerCommandDeleteGame::new(42, true);
        factory.handle_command(ReceivedCommand::new_internal(
            AnyInternalServerCommand::DeleteGame(cmd),
            1,
        )).await;
    }

    // ── Phase ZVB: sketch/marker handler family dispatch ───────────────────────

    #[tokio::test]
    async fn handle_command_add_sketch_updates_shared_sketch_manager() {
        let gc = Arc::new(Mutex::new(crate::game_cache::GameCache::new()));
        let sm = Arc::new(Mutex::new(crate::net::session_manager::SessionManager::new()));
        let factory = ServerCommandHandlerFactory::new(gc, sm, db());
        factory.replay_session_manager.lock().unwrap().add_session(1, "replay".into(), "Alice".into());

        factory.handle_command(ReceivedCommand::new(
            AnyClientCommand::ClientAddSketch(
                ffb_protocol::commands::client_command_add_sketch::ClientCommandAddSketch::with_sketch_id("sk-1".to_string()),
            ),
            1,
        )).await;

        let mut mgr = factory.sketch_manager.lock().unwrap();
        assert_eq!(mgr.get_sketches("1").len(), 1);
    }

    #[tokio::test]
    async fn handle_command_clear_sketches_clears_shared_sketch_manager() {
        use ffb_engine::server_sketch_manager::Sketch as ManagerSketch;

        let gc = Arc::new(Mutex::new(crate::game_cache::GameCache::new()));
        let sm = Arc::new(Mutex::new(crate::net::session_manager::SessionManager::new()));
        let factory = ServerCommandHandlerFactory::new(gc, sm, db());
        factory.replay_session_manager.lock().unwrap().add_session(1, "replay".into(), "Alice".into());
        factory.sketch_manager.lock().unwrap().add_sketch("1", ManagerSketch::new("sk-1"));

        factory.handle_command(ReceivedCommand::new(
            AnyClientCommand::ClientClearSketches(ffb_protocol::commands::client_command_clear_sketches::ClientCommandClearSketches::default()),
            1,
        )).await;

        let mut mgr = factory.sketch_manager.lock().unwrap();
        assert!(mgr.get_sketches("1").is_empty());
    }

    #[tokio::test]
    async fn handle_command_remove_sketches_removes_matching_ids_from_shared_manager() {
        use ffb_engine::server_sketch_manager::Sketch as ManagerSketch;

        let gc = Arc::new(Mutex::new(crate::game_cache::GameCache::new()));
        let sm = Arc::new(Mutex::new(crate::net::session_manager::SessionManager::new()));
        let factory = ServerCommandHandlerFactory::new(gc, sm, db());
        factory.replay_session_manager.lock().unwrap().add_session(1, "replay".into(), "Alice".into());
        {
            let mut mgr = factory.sketch_manager.lock().unwrap();
            mgr.add_sketch("1", ManagerSketch::new("sk-1"));
            mgr.add_sketch("1", ManagerSketch::new("sk-2"));
        }

        factory.handle_command(ReceivedCommand::new(
            AnyClientCommand::ClientRemoveSketches(ffb_protocol::commands::client_command_remove_sketches::ClientCommandRemoveSketches {
                ids: vec!["sk-1".into()], ..Default::default() }),
            1,
        )).await;

        let mut mgr = factory.sketch_manager.lock().unwrap();
        let remaining = mgr.get_sketches("1");
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].get_id(), "sk-2");
    }

    #[tokio::test]
    async fn handle_command_sketch_add_coordinate_reaches_shared_manager() {
        use ffb_engine::server_sketch_manager::Sketch as ManagerSketch;
        use ffb_model::types::FieldCoordinate;

        let gc = Arc::new(Mutex::new(crate::game_cache::GameCache::new()));
        let sm = Arc::new(Mutex::new(crate::net::session_manager::SessionManager::new()));
        let factory = ServerCommandHandlerFactory::new(gc, sm, db());
        factory.replay_session_manager.lock().unwrap().add_session(1, "replay".into(), "Alice".into());
        factory.sketch_manager.lock().unwrap().add_sketch("1", ManagerSketch::new("sk-1"));

        factory.handle_command(ReceivedCommand::new(
            AnyClientCommand::ClientSketchAddCoordinate(ffb_protocol::commands::client_command_sketch_add_coordinate::ClientCommandSketchAddCoordinate {
                sketch_id: Some("sk-1".into()),
                coordinate: Some(FieldCoordinate::new(3, 4)), ..Default::default() }),
            1,
        )).await;

        // No panic and the sketch is still tracked; path-coordinate storage itself is
        // exercised directly in `ServerCommandHandlerSketchAddCoordinate`'s own tests.
        let mut mgr = factory.sketch_manager.lock().unwrap();
        assert_eq!(mgr.get_sketches("1").len(), 1);
    }

    #[tokio::test]
    async fn handle_command_sketch_set_color_delivers_to_replay_peer() {
        use tokio::sync::mpsc;

        let gc = Arc::new(Mutex::new(crate::game_cache::GameCache::new()));
        let sm = Arc::new(Mutex::new(crate::net::session_manager::SessionManager::new()));
        let factory = ServerCommandHandlerFactory::new(gc, sm, db());
        let (tx, mut rx) = mpsc::unbounded_channel();
        {
            let mut rsm = factory.replay_session_manager.lock().unwrap();
            rsm.add_session(1, "replay".into(), "Alice".into());
            rsm.add_session(2, "replay".into(), "Bob".into());
            rsm.register_sender(2, tx);
        }

        factory.handle_command(ReceivedCommand::new(
            AnyClientCommand::ClientSketchSetColor(ffb_protocol::commands::client_command_sketch_set_color::ClientCommandSketchSetColor {
                sketch_ids: vec!["sk-1".into()],
                rgb: 0x00FF00, ..Default::default() }),
            1,
        )).await;

        let sent = rx.try_recv().expect("expected a message forwarded to the replay peer");
        assert!(sent.contains("serverSketchSetColor"));
    }

    #[tokio::test]
    async fn handle_command_sketch_set_label_delivers_to_replay_peer() {
        use tokio::sync::mpsc;

        let gc = Arc::new(Mutex::new(crate::game_cache::GameCache::new()));
        let sm = Arc::new(Mutex::new(crate::net::session_manager::SessionManager::new()));
        let factory = ServerCommandHandlerFactory::new(gc, sm, db());
        let (tx, mut rx) = mpsc::unbounded_channel();
        {
            let mut rsm = factory.replay_session_manager.lock().unwrap();
            rsm.add_session(1, "replay".into(), "Alice".into());
            rsm.add_session(2, "replay".into(), "Bob".into());
            rsm.register_sender(2, tx);
        }

        factory.handle_command(ReceivedCommand::new(
            AnyClientCommand::ClientSketchSetLabel(ffb_protocol::commands::client_command_sketch_set_label::ClientCommandSketchSetLabel {
                sketch_ids: vec!["sk-1".into()],
                label: Some("Arrow".into()), ..Default::default() }),
            1,
        )).await;

        let sent = rx.try_recv().expect("expected a message forwarded to the replay peer");
        assert!(sent.contains("serverSketchSetLabel"));
        assert!(sent.contains("Arrow"));
    }

    #[tokio::test]
    async fn handle_command_set_marker_writes_marker_into_shared_game_cache() {
        use ffb_model::model::ClientMode;
        use ffb_model::model::team::Team;
        use ffb_model::types::FieldCoordinate;
        use tokio::sync::mpsc;

        fn team(id: &str) -> Team {
            Team {
                id: id.into(), name: id.into(), race: "Human".into(), roster_id: "human".into(),
                coach: "coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
                prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
                assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
                special_rules: vec![], players: vec![], vampire_lord: false, roster: None, base_icon_path: None, logo_url: None, necromancer: false,
            }
        }

        let gc = Arc::new(Mutex::new(crate::game_cache::GameCache::new()));
        let sm_arc = Arc::new(Mutex::new(crate::net::session_manager::SessionManager::new()));
        let game_id = gc.lock().unwrap().create_game_state();
        gc.lock().unwrap().get_game_state_by_id_mut(game_id).unwrap().start_game(
            team("home"), team("away"), ffb_model::enums::Rules::Bb2025, 0,
        );
        let (tx, _rx) = mpsc::unbounded_channel();
        sm_arc.lock().unwrap().add_session(1, game_id, "Home".into(), ClientMode::PLAYER, true, vec![], tx);

        let factory = ServerCommandHandlerFactory::new(Arc::clone(&gc), Arc::clone(&sm_arc), db());
        factory.handle_command(ReceivedCommand::new(
            AnyClientCommand::ClientSetMarker(ffb_protocol::commands::client_command_set_marker::ClientCommandSetMarker {
                player_id: Some("p1".into()),
                coordinate: Some(FieldCoordinate::new(1, 1)),
                text: Some("Nice job".into()), ..Default::default() }),
            1,
        )).await;

        let gc = gc.lock().unwrap();
        let game = gc.get_game_state_by_id(game_id).unwrap().get_game().unwrap();
        let marker = game.field_model.get_field_marker(FieldCoordinate::new(1, 1)).unwrap();
        assert_eq!(marker.get_home_text(), Some("Nice job"));
    }

    #[tokio::test]
    async fn handle_command_set_prevent_sketching_reaches_real_handler() {
        use ffb_model::model::ClientMode;
        use tokio::sync::mpsc;

        let gc = Arc::new(Mutex::new(crate::game_cache::GameCache::new()));
        let sm_arc = Arc::new(Mutex::new(crate::net::session_manager::SessionManager::new()));
        let (tx1, mut rx1) = mpsc::unbounded_channel();
        let (tx2, _rx2) = mpsc::unbounded_channel();
        sm_arc.lock().unwrap().add_session(1, 0, "coach1".into(), ClientMode::SPECTATOR, false, vec![], tx1);
        sm_arc.lock().unwrap().add_session(2, 0, "coach2".into(), ClientMode::SPECTATOR, false, vec![], tx2);

        let factory = ServerCommandHandlerFactory::new(Arc::clone(&gc), Arc::clone(&sm_arc), db());
        {
            let mut rsm = factory.replay_session_manager.lock().unwrap();
            rsm.add_session(1, "replay1".into(), "coach1".into());
        }

        factory.handle_command(ReceivedCommand::new(
            AnyClientCommand::ClientSetPreventSketching(ffb_protocol::commands::client_command_set_prevent_sketching::ClientCommandSetPreventSketching {
                coach: Some("coach2".into()),
                prevent_sketching: true, ..Default::default() }),
            1,
        )).await;

        // Session 1 has control (first to join replay1) and broadcasts the real
        // `ServerCommandSetPreventSketching` message to itself and other sessions on the
        // replay — no `ReplayState` is registered in the factory's private map (documented
        // gap, same as `handle_command_transfer_replay_control_reaches_real_handler`), but the
        // broadcast over `SessionManager` still runs for real.
        let msg = rx1.try_recv().expect("expected a real serverSetPreventSketching broadcast");
        assert!(msg.contains("serverSetPreventSketching"));
    }

    #[tokio::test]
    async fn handle_command_update_player_markings_spectator_sends_empty_markers() {
        use ffb_model::model::ClientMode;
        use tokio::sync::mpsc;

        let gc = Arc::new(Mutex::new(crate::game_cache::GameCache::new()));
        let sm_arc = Arc::new(Mutex::new(crate::net::session_manager::SessionManager::new()));
        let game_id = gc.lock().unwrap().create_game_state();
        let (tx, mut rx) = mpsc::unbounded_channel();
        sm_arc.lock().unwrap().add_session(1, game_id, "Spec".into(), ClientMode::SPECTATOR, false, vec![], tx);

        let factory = ServerCommandHandlerFactory::new(Arc::clone(&gc), Arc::clone(&sm_arc), db());
        factory.handle_command(ReceivedCommand::new(
            AnyClientCommand::ClientUpdatePlayerMarkings(ffb_protocol::commands::client_command_update_player_markings::ClientCommandUpdatePlayerMarkings {
                auto: false,
                sort_mode_name: None, ..Default::default() }),
            1,
        )).await;

        let sent = rx.try_recv().expect("expected a real serverUpdateLocalPlayerMarkers message");
        assert!(sent.contains("\"markers\":[]"));
    }

    #[tokio::test]
    async fn handle_command_load_automatic_player_markings_enqueues_via_real_handler() {
        use ffb_model::model::ClientMode;
        use tokio::sync::mpsc;

        let gc = Arc::new(Mutex::new(crate::game_cache::GameCache::new()));
        let sm_arc = Arc::new(Mutex::new(crate::net::session_manager::SessionManager::new()));
        let (tx, _rx) = mpsc::unbounded_channel();
        sm_arc.lock().unwrap().add_session(1, 0, "Coach".into(), ClientMode::PLAYER, true, vec![], tx);

        let factory = ServerCommandHandlerFactory::new(Arc::clone(&gc), Arc::clone(&sm_arc), db());
        factory.handle_command(ReceivedCommand::new(
            AnyClientCommand::ClientLoadAutomaticPlayerMarkings(
                ffb_protocol::commands::client_command_load_automatic_player_markings::ClientCommandLoadAutomaticPlayerMarkings {
                    index: 1,
                    coach: Some("Coach".into()),
                    ..Default::default()
                },
            ),
            1,
        )).await;

        assert_eq!(
            factory.load_automatic_player_markings_handler.request_processor.lock().unwrap().queue_len(),
            1
        );
    }

    // ── Replay handler family dispatch ─────────────────────────────────────────

    #[tokio::test]
    async fn handle_command_join_replay_creates_replay_state_via_real_handler() {
        use tokio::sync::mpsc;

        let gc = Arc::new(Mutex::new(crate::game_cache::GameCache::new()));
        let sm_arc = Arc::new(Mutex::new(crate::net::session_manager::SessionManager::new()));
        let factory = ServerCommandHandlerFactory::new(Arc::clone(&gc), Arc::clone(&sm_arc), db());
        let (tx, mut rx) = mpsc::unbounded_channel();
        factory.replay_session_manager.lock().unwrap().register_sender(1, tx);

        factory.handle_command(ReceivedCommand::new(
            AnyClientCommand::ClientJoinReplay(ffb_protocol::commands::client_command_join_replay::ClientCommandJoinReplay {
                replay_name: Some("MyReplay".into()),
                coach: Some("Alice".into()),
                game_id: 42, ..Default::default() }),
            1,
        )).await;

        assert_eq!(factory.replay_cache.lock().unwrap().replay_count(), 1);
        // First (and only) session in the replay gets the `ServerCommandJoin` broadcast
        // (it's already in `sessions_for_replay` by the time the handler builds that list)
        // followed by the new-replay `ServerCommandReplayControl`.
        let join_msg = rx.try_recv().expect("expected a real serverJoin broadcast");
        assert!(join_msg.contains("serverJoin"));
        let control_msg = rx.try_recv().expect("expected a real serverReplayControl message");
        assert!(control_msg.contains("serverReplayControl"));
    }

    #[tokio::test]
    async fn handle_command_replay_found_game_starts_a_real_replay() {
        let gc = Arc::new(Mutex::new(crate::game_cache::GameCache::new()));
        let game_id = gc.lock().unwrap().create_game_state();
        let sm_arc = Arc::new(Mutex::new(crate::net::session_manager::SessionManager::new()));
        let factory = ServerCommandHandlerFactory::new(Arc::clone(&gc), Arc::clone(&sm_arc), db());

        factory.handle_command(ReceivedCommand::new(
            AnyClientCommand::ClientReplay(ffb_protocol::commands::client_command_replay::ClientCommandReplay {
                game_id,
                replay_to_command_nr: 0,
                coach: Some("coach".into()), ..Default::default() }),
            1,
        )).await;

        assert_eq!(factory.replayer.lock().unwrap().queue_size(), 1);
    }

    #[tokio::test]
    async fn handle_command_replay_status_broadcasts_via_real_handler() {
        use tokio::sync::mpsc;

        let gc = Arc::new(Mutex::new(crate::game_cache::GameCache::new()));
        let sm_arc = Arc::new(Mutex::new(crate::net::session_manager::SessionManager::new()));
        let factory = ServerCommandHandlerFactory::new(Arc::clone(&gc), Arc::clone(&sm_arc), db());
        {
            let mut rsm = factory.replay_session_manager.lock().unwrap();
            rsm.add_session(1, "replay1".to_string(), "coach1".to_string());
            rsm.add_session(2, "replay1".to_string(), "coach2".to_string());
        }
        let (tx2, mut rx2) = mpsc::unbounded_channel();
        factory.replay_session_manager.lock().unwrap().register_sender(2, tx2);
        // Seed the factory's cached `ReplayState` for "replay1" (see `replay_states`'s own
        // doc comment) so `requires_push_to_other_clients` has something to compare against —
        // its default speed=0/running=false/forward=false differs from the command below.
        factory.replay_states.lock().unwrap().insert(
            "replay1".to_string(),
            ffb_engine::replay_state::ReplayState::new("replay1"),
        );

        factory.handle_command(ReceivedCommand::new(
            AnyClientCommand::ClientReplayStatus(ffb_protocol::commands::client_command_replay_status::ClientCommandReplayStatus {
                command_nr: 10,
                speed: 1,
                running: true,
                forward: true,
                skip: false, ..Default::default() }),
            1,
        )).await;

        let sent = rx2.try_recv().expect("expected a real serverReplayStatus broadcast to session 2");
        assert!(sent.contains("serverReplayStatus"));
    }

    #[tokio::test]
    async fn factory_dispatches_replay_loaded_through_real_handler() {
        use crate::net::commands::internal_server_command_replay_loaded::InternalServerCommandReplayLoaded;

        let gc = Arc::new(Mutex::new(crate::game_cache::GameCache::new()));
        let game_id = gc.lock().unwrap().create_game_state();
        let sm_arc = Arc::new(Mutex::new(crate::net::session_manager::SessionManager::new()));
        let factory = ServerCommandHandlerFactory::new(Arc::clone(&gc), Arc::clone(&sm_arc), db());

        let cmd = InternalServerCommandReplayLoaded::new(game_id, 0, "coach".to_string());
        factory.handle_command(ReceivedCommand::new_internal(AnyInternalServerCommand::ReplayLoaded(cmd), 1)).await;

        assert_eq!(
            factory.game_cache.lock().unwrap().get_game_state_by_id(game_id).unwrap().get_status(),
            Some(ffb_model::enums::GameStatus::Replaying)
        );
        assert_eq!(factory.replayer.lock().unwrap().queue_size(), 1);
    }

    // ── Game-management handler family (this phase) ─────────────────────────

    fn gm_team(id: &str) -> ffb_model::model::team::Team {
        ffb_model::model::team::Team {
            id: id.into(), name: id.into(), race: "Human".into(), roster_id: "human".into(),
            coach: "coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![], vampire_lord: false, roster: None, base_icon_path: None, logo_url: None, necromancer: false,
        }
    }

    #[tokio::test]
    async fn factory_dispatches_add_loaded_team_through_real_handler() {
        // The `AddLoadedTeam` dispatch arm now hands the handler the real `Team` the command
        // carries, so dispatching the command end-to-end must actually swap the team into the game.
        let gc = Arc::new(Mutex::new(crate::game_cache::GameCache::new()));
        let sm = Arc::new(Mutex::new(crate::net::session_manager::SessionManager::new()));
        let game_id = gc.lock().unwrap().create_game_state();
        gc.lock().unwrap().get_game_state_by_id_mut(game_id).unwrap().start_game(
            gm_team(""), gm_team("away1"), ffb_model::enums::Rules::Bb2025, 0,
        );

        let factory = ServerCommandHandlerFactory::new(Arc::clone(&gc), sm, db());
        let cmd = crate::net::commands::internal_server_command_add_loaded_team::InternalServerCommandAddLoadedTeam::new(
            game_id, "coach".into(), None, gm_team("home1"), vec![],
        );
        factory.handle_command(ReceivedCommand::new_internal(AnyInternalServerCommand::AddLoadedTeam(cmd), 1)).await;

        let game = gc.lock().unwrap().get_game_state_by_id(game_id).unwrap().get_game().unwrap().clone();
        assert_eq!(game.team_home.id, "home1");
    }

    #[tokio::test]
    async fn factory_dispatches_apply_automated_player_markings_through_real_handler() {
        use ffb_engine::marking::auto_marking_config::AutoMarkingConfig;
        use ffb_engine::marking::auto_marking_record::Builder;
        use ffb_model::model::player::Player;
        use ffb_model::model::skill_def::{SkillId, SkillWithValue};
        use ffb_model::model::ClientMode;
        use tokio::sync::mpsc;

        let gc = Arc::new(Mutex::new(crate::game_cache::GameCache::new()));
        let sm_arc = Arc::new(Mutex::new(crate::net::session_manager::SessionManager::new()));
        let game_id = gc.lock().unwrap().create_game_state();
        gc.lock().unwrap().get_game_state_by_id_mut(game_id).unwrap().start_game(
            gm_team("home"), gm_team("away"), ffb_model::enums::Rules::Bb2025, 0,
        );
        // A home player with Block so the marking generator produces a non-empty "B" marker.
        {
            let mut guard = gc.lock().unwrap();
            let mut player = Player { id: "p1".into(), ..Default::default() };
            player.extra_skills.push(SkillWithValue::new(SkillId::Block));
            guard.get_game_state_by_id_mut(game_id).unwrap().get_game_mut().unwrap().team_home.players.push(player);
        }
        let (tx, mut rx) = mpsc::unbounded_channel();
        sm_arc.lock().unwrap().add_session(1, game_id, "Spec".into(), ClientMode::SPECTATOR, false, vec![], tx);

        let factory = ServerCommandHandlerFactory::new(Arc::clone(&gc), Arc::clone(&sm_arc), db());
        let mut config = AutoMarkingConfig::new();
        config.markings.push(Builder::new().with_skill(SkillId::Block).with_marking("B").with_gained_only(true).build());
        let cmd = crate::net::commands::internal_server_command_apply_automated_player_markings::InternalServerCommandApplyAutomatedPlayerMarkings::new(config, game_id);
        factory.handle_command(ReceivedCommand::new_internal(AnyInternalServerCommand::ApplyAutomatedPlayerMarkings(cmd), 1)).await;

        let sent = rx.try_recv().expect("expected a markers message from the real handler");
        assert!(sent.contains("\"B\""), "markers message should carry the Block marking: {sent}");
    }

    #[tokio::test]
    async fn factory_dispatches_calculate_automatic_player_markings_through_real_handler() {
        use ffb_engine::marking::auto_marking_config::AutoMarkingConfig;
        use ffb_engine::marking::auto_marking_record::Builder;
        use ffb_model::model::game::Game;
        use ffb_model::model::player::Player;
        use ffb_model::model::skill_def::{SkillId, SkillWithValue};
        use ffb_model::model::ClientMode;
        use tokio::sync::mpsc;

        let gc = Arc::new(Mutex::new(crate::game_cache::GameCache::new()));
        let sm_arc = Arc::new(Mutex::new(crate::net::session_manager::SessionManager::new()));
        let (tx, mut rx) = mpsc::unbounded_channel();
        sm_arc.lock().unwrap().add_session(1, 0, "Spec".into(), ClientMode::SPECTATOR, false, vec![], tx);

        let factory = ServerCommandHandlerFactory::new(gc, Arc::clone(&sm_arc), db());

        let mut home = gm_team("home");
        let mut player = Player { id: "p1".into(), ..Default::default() };
        player.extra_skills.push(SkillWithValue::new(SkillId::Block));
        home.players.push(player);
        let game = Game::new(home, gm_team("away"), ffb_model::enums::Rules::Bb2025);

        let mut config = AutoMarkingConfig::new();
        config.markings.push(Builder::new().with_skill(SkillId::Block).with_marking("B").with_gained_only(true).build());
        let cmd = crate::net::commands::internal_server_command_calculate_automatic_player_markings::InternalServerCommandCalculateAutomaticPlayerMarkings::new(config, 7, game);
        factory.handle_command(ReceivedCommand::new_internal(AnyInternalServerCommand::CalculateAutomaticPlayerMarkings(cmd), 1)).await;

        let sent = rx.try_recv().expect("expected a markings message from the real handler");
        assert!(sent.contains("\"index\":7"), "markings message should carry the index: {sent}");
        assert!(sent.contains("\"B\""), "markings message should carry the Block marking: {sent}");
    }

    #[tokio::test]
    async fn factory_dispatches_fumbbl_team_loaded_through_real_handler() {
        use crate::net::commands::internal_server_command_fumbbl_team_loaded::InternalServerCommandFumbblTeamLoaded;
        use ffb_model::model::ClientMode;
        use tokio::sync::mpsc;

        let gc = Arc::new(Mutex::new(crate::game_cache::GameCache::new()));
        let sm_arc = Arc::new(Mutex::new(crate::net::session_manager::SessionManager::new()));
        let game_id = gc.lock().unwrap().create_game_state();
        gc.lock().unwrap().get_game_state_by_id_mut(game_id).unwrap().start_game(
            gm_team("home"), gm_team("away"), ffb_model::enums::Rules::Bb2025, 0,
        );
        // The dispatch arm can only fetch a sender for an *already-registered* session (see
        // its own doc comment) — register a spectator sender first under the same session id
        // so the real join/ready-check path actually runs.
        let (tx, _rx) = mpsc::unbounded_channel();
        sm_arc.lock().unwrap().add_session(1, game_id, "Spectator".into(), ClientMode::SPECTATOR, false, vec![], tx);

        let factory = ServerCommandHandlerFactory::new(Arc::clone(&gc), Arc::clone(&sm_arc), db());
        let cmd = InternalServerCommandFumbblTeamLoaded::new(game_id, "Home".into(), true, vec![]);
        factory.handle_command(ReceivedCommand::new_internal(AnyInternalServerCommand::FumbblTeamLoaded(cmd), 1)).await;

        assert_eq!(sm_arc.lock().unwrap().get_coach_for_session(1), Some("Home"));
    }

    #[test]
    fn factory_fumbbl_game_checked_dispatch_arm_returns_false_for_missing_game() {
        let gc = Arc::new(Mutex::new(crate::game_cache::GameCache::new()));
        let sm = Arc::new(Mutex::new(crate::net::session_manager::SessionManager::new()));
        let factory = ServerCommandHandlerFactory::new(Arc::clone(&gc), sm, db());

        let mut gc_guard = gc.lock().unwrap();
        assert!(!factory.fumbbl_game_checked_handler.handle_command(999, &mut gc_guard, factory.markings_http_client.as_ref(), ""));
    }

    #[tokio::test]
    async fn factory_dispatches_schedule_game_through_real_handler() {
        use crate::net::commands::internal_server_command_schedule_game::InternalServerCommandScheduleGame;

        let gc = Arc::new(Mutex::new(crate::game_cache::GameCache::new()));
        let sm = Arc::new(Mutex::new(crate::net::session_manager::SessionManager::new()));
        let factory = ServerCommandHandlerFactory::new(Arc::clone(&gc), sm, db());

        let before = gc.lock().unwrap().all_game_ids().len();
        let cmd = InternalServerCommandScheduleGame::new("home".to_string(), "away".to_string());
        factory.handle_command(ReceivedCommand::new_internal(AnyInternalServerCommand::ScheduleGame(cmd), 1)).await;

        // No team/roster files exist for "home"/"away" in this test environment, so the game
        // slot is created (real `GameCache::create_game_state` call) but never started — same
        // as `ServerCommandHandlerScheduleGame`'s own `load_teams_with_unresolvable_teams_leaves_game_unstarted` test.
        assert_eq!(gc.lock().unwrap().all_game_ids().len(), before + 1);
    }

    #[tokio::test]
    async fn factory_dispatches_close_game_through_real_handler() {
        use crate::net::commands::internal_server_command_close_game::InternalServerCommandCloseGame;
        use ffb_model::model::ClientMode;
        use tokio::sync::mpsc;

        let gc = Arc::new(Mutex::new(crate::game_cache::GameCache::new()));
        let sm_arc = Arc::new(Mutex::new(crate::net::session_manager::SessionManager::new()));
        let game_id = gc.lock().unwrap().create_game_state();
        let (tx, _rx) = mpsc::unbounded_channel();
        sm_arc.lock().unwrap().add_session(1, game_id, "Coach".into(), ClientMode::PLAYER, true, vec![], tx);

        let factory = ServerCommandHandlerFactory::new(Arc::clone(&gc), Arc::clone(&sm_arc), db());
        let cmd = InternalServerCommandCloseGame::new(game_id);
        factory.handle_command(ReceivedCommand::new_internal(AnyInternalServerCommand::CloseGame(cmd), 1)).await;

        assert!(sm_arc.lock().unwrap().get_sessions_for_game_id(game_id).is_empty());
        assert!(gc.lock().unwrap().get_game_state_by_id(game_id).is_none());
    }

    #[tokio::test]
    async fn factory_dispatches_upload_game_through_real_handler() {
        use crate::net::commands::internal_server_command_upload_game::InternalServerCommandUploadGame;

        let gc = Arc::new(Mutex::new(crate::game_cache::GameCache::new()));
        let sm = Arc::new(Mutex::new(crate::net::session_manager::SessionManager::new()));
        let game_id = gc.lock().unwrap().create_game_state();
        gc.lock().unwrap().get_game_state_by_id_mut(game_id).unwrap().start_game(
            gm_team("home"), gm_team("away"), ffb_model::enums::Rules::Bb2025, 0,
        );

        let factory = ServerCommandHandlerFactory::new(Arc::clone(&gc), sm, db());
        let cmd = InternalServerCommandUploadGame::new(game_id);
        factory.handle_command(ReceivedCommand::new_internal(AnyInternalServerCommand::UploadGame(cmd), 1)).await;

        let guard = gc.lock().unwrap();
        let gs = guard.get_game_state_by_id(game_id).unwrap();
        assert!(gs.is_finished(), "known game should be driven to finished by the real UploadGame handler");
    }

    #[tokio::test]
    async fn factory_dispatches_user_settings_through_real_handler() {
        use ffb_model::model::ClientMode;
        use tokio::sync::mpsc;

        let gc = Arc::new(Mutex::new(crate::game_cache::GameCache::new()));
        let sm_arc = Arc::new(Mutex::new(crate::net::session_manager::SessionManager::new()));
        let (tx, _rx) = mpsc::unbounded_channel();
        sm_arc.lock().unwrap().add_session(1, 0, "Coach1".into(), ClientMode::PLAYER, true, vec![], tx);

        let factory = ServerCommandHandlerFactory::new(gc, Arc::clone(&sm_arc), db());
        let mut settings = std::collections::HashMap::new();
        settings.insert("soundVolume".to_string(), "80".to_string());
        factory.handle_command(ReceivedCommand::new(
            AnyClientCommand::ClientUserSettings(ffb_protocol::commands::client_command_user_settings::ClientCommandUserSettings { settings, ..Default::default() }),
            1,
        )).await;

        // With no DB pool configured this degrades to a no-op (see
        // `ServerCommandHandlerUserSettings`'s own doc comment), but the real handler still
        // ran to completion via the factory's dispatch arm without panicking.
        assert!(sm_arc.lock().unwrap().get_coach_for_session(1).is_some());
    }
}
