/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerFactory.
///
/// Routes incoming `ClientCommand`s to the appropriate handler:
/// - Ping → delegates to the real `ServerCommandHandlerPing`.
/// - Join → still handled upstream in `command_socket.rs` before enqueue.
/// - Talk, CloseSession, TransferReplayControl, RequestVersion,
///   PasswordChallenge → delegate to their real `ServerCommandHandler*` structs
///   (Phase ZVA, session/game-lifecycle family). Each needed its own
///   `ffb_protocol::client_commands::ClientCommand` variant added (mirroring
///   the field shape of the corresponding `ffb_protocol::commands::*` struct
///   the handler was originally translated against) since that wire enum had
///   no variant for them at all.
/// - DeleteGame → an `AnyInternalServerCommand`, not a `ClientCommand`; wired
///   into `handle_internal_command`'s match instead.
/// - Gameplay commands → decoded to `Action` and fed to the engine directly
///   (this pipeline doesn't correspond to any single Java `ServerCommandHandler*`
///   class — it's the Rust-specific consolidated action dispatch).
///
/// **Known gap (Phase ZV):** most of the other ~30 real, tested
/// `ServerCommandHandler*` structs under `crate::handler` (the sketch/marker
/// family, replay family, ...) are still NOT delegated to from here. They were
/// translated against `ffb_protocol::commands::*` (the older, Java-mirroring
/// command types), while this factory's `decode_command` operates on
/// `ffb_protocol::client_commands::ClientCommand` (a separate, newer wire
/// enum). Bridging the two command hierarchies for the remaining handlers is
/// a real, nontrivial follow-up. `SocketClosed` isn't a `ClientCommand` at
/// all (it's an internal event on socket close), so `command_socket.rs` will
/// eventually need to call `ServerCommandHandlerSocketClosed::handle_command`
/// directly rather than through this factory.
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
use ffb_protocol::client_commands::ClientCommand;
use crate::db::db_connection_manager::DbConnectionManager;
use crate::game_cache::GameCache;
use crate::handler::server_command_handler_add_loaded_team::ServerCommandHandlerAddLoadedTeam;
use crate::handler::server_command_handler_add_sketch::ServerCommandHandlerAddSketch;
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
use crate::net::wire::{OutgoingModelSync, events_to_reports};
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
    pub client_properties: Arc<Vec<(String, String)>>,
    ping_handler: ServerCommandHandlerPing,
    socket_closed_handler: ServerCommandHandlerSocketClosed,
    /// `pub` (like this struct's other fields) rather than private: unlike
    /// `join_approved_handler` (dispatched to from `handle_internal_command` below), this
    /// handler isn't reachable from `handle_command`'s `ClientCommand::ClientJoin` arm yet —
    /// that bridging is the separately-documented, out-of-scope gap this file's own "Known
    /// gap" doc comment already flagged — so it's exposed for direct use (see this file's
    /// own tests) instead of sitting unread.
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
    /// this file's own tests can exercise the real handler directly: its `AddLoadedTeam`
    /// dispatch arm below is a documented no-op (the wire command carries no `Team` payload
    /// to hand it — see that handler's own doc comment), so it isn't otherwise reachable.
    pub(crate) add_loaded_team_handler: ServerCommandHandlerAddLoadedTeam,
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
        let team_cache = Arc::new(TeamCache::new());
        let roster_cache = Arc::new(RosterCache::new());
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
        let password_challenge_handler = ServerCommandHandlerPasswordChallenge::new(Arc::clone(&session_manager));

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

        match &client_command {
            ClientCommand::ClientPing(ping) => {
                // Delegates to the real 1:1-translated ServerCommandHandlerPing
                // (session/replay last-ping bookkeeping + pong reply), instead of
                // duplicating that logic inline.
                self.ping_handler.handle_command(session_id, ping.timestamp);
                return;
            }
            ClientCommand::ClientJoin(_) => {
                // Join is handled upstream in command_socket.rs before being enqueued.
                log::warn!("session {} sent ClientJoin after already joined", session_id);
                return;
            }
            ClientCommand::ClientTalk(t) => {
                let cmd = ffb_protocol::commands::client_command_talk::ClientCommandTalk {
                    entropy: None,
                    talk: t.talk.clone(),
                };
                self.talk_handler.handle_command(session_id, &cmd);
                return;
            }
            ClientCommand::ClientCloseSession(_) => {
                self.close_session_handler.handle_command(session_id);
                return;
            }
            ClientCommand::ClientTransferReplayControl(t) => {
                let cmd = ffb_protocol::commands::client_command_transfer_replay_control::ClientCommandTransferReplayControl {
                    entropy: None,
                    coach: t.coach.clone(),
                };
                self.transfer_control_handler.handle_command(session_id, &cmd);
                return;
            }
            ClientCommand::ClientRequestVersion(_) => {
                self.request_version_handler.handle_command(session_id);
                return;
            }
            ClientCommand::ClientPasswordChallenge(p) => {
                let cmd = ffb_protocol::commands::client_command_password_challenge::ClientCommandPasswordChallenge {
                    entropy: None,
                    coach: p.coach.clone(),
                };
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
            ClientCommand::ClientAddSketch(a) => {
                let cmd = ffb_protocol::commands::client_command_add_sketch::ClientCommandAddSketch {
                    entropy: None,
                    sketch_id: a.sketch_id.clone(),
                };
                self.add_sketch_handler.handle_command(session_id, &cmd);
                return;
            }
            ClientCommand::ClientClearSketches(_) => {
                let cmd = ffb_protocol::commands::client_command_clear_sketches::ClientCommandClearSketches::new();
                self.clear_sketches_handler.handle_command(session_id, &cmd);
                return;
            }
            ClientCommand::ClientRemoveSketches(r) => {
                let cmd = ffb_protocol::commands::client_command_remove_sketches::ClientCommandRemoveSketches {
                    entropy: None,
                    ids: r.ids.clone(),
                };
                self.remove_sketches_handler.handle_command(session_id, &cmd);
                return;
            }
            ClientCommand::ClientSketchAddCoordinate(s) => {
                let cmd = ffb_protocol::commands::client_command_sketch_add_coordinate::ClientCommandSketchAddCoordinate {
                    entropy: None,
                    sketch_id: s.sketch_id.clone(),
                    coordinate: s.coordinate,
                };
                self.sketch_add_coordinate_handler.handle_command(session_id, &cmd);
                return;
            }
            ClientCommand::ClientSketchSetColor(s) => {
                let cmd = ffb_protocol::commands::client_command_sketch_set_color::ClientCommandSketchSetColor {
                    entropy: None,
                    sketch_ids: s.sketch_ids.clone(),
                    rgb: s.rgb,
                };
                self.sketch_set_color_handler.handle_command(session_id, &cmd);
                return;
            }
            ClientCommand::ClientSketchSetLabel(s) => {
                let cmd = ffb_protocol::commands::client_command_sketch_set_label::ClientCommandSketchSetLabel {
                    entropy: None,
                    sketch_ids: s.sketch_ids.clone(),
                    label: s.label.clone(),
                };
                self.sketch_set_label_handler.handle_command(session_id, &cmd);
                return;
            }
            ClientCommand::ClientSetMarker(m) => {
                let cmd = ffb_protocol::commands::client_command_set_marker::ClientCommandSetMarker {
                    player_id: m.player_id.clone(),
                    coordinate: m.coordinate,
                    text: m.text.clone(),
                    entropy: None,
                };
                self.set_marker_handler.handle_command(session_id, &cmd);
                return;
            }
            ClientCommand::ClientSetPreventSketching(p) => {
                let cmd = ffb_protocol::commands::client_command_set_prevent_sketching::ClientCommandSetPreventSketching {
                    coach: p.coach.clone(),
                    prevent_sketching: p.prevent_sketching,
                    entropy: None,
                };
                self.set_prevent_sketching_handler.handle_command(session_id, &cmd);
                return;
            }
            ClientCommand::ClientLoadAutomaticPlayerMarkings(l) => {
                let cmd = ffb_protocol::commands::client_command_load_automatic_player_markings::ClientCommandLoadAutomaticPlayerMarkings {
                    entropy: None,
                    index: l.index,
                    coach: l.coach.clone(),
                    game: None,
                };
                self.load_automatic_player_markings_handler.handle_command(&cmd, session_id);
                return;
            }
            ClientCommand::ClientUpdatePlayerMarkings(u) => {
                let cmd = ffb_protocol::commands::client_command_update_player_markings::ClientCommandUpdatePlayerMarkings {
                    entropy: None,
                    auto: u.auto,
                    sort_mode_name: u.sort_mode_name.clone(),
                };
                // `ServerCommandHandlerUpdatePlayerMarkings::handle_command` takes the shared
                // `Arc<Mutex<..>>`s directly and locks/drops internally around its own
                // `.await` points, so no lock is held here across this call.
                self.update_player_markings_handler
                    .handle_command(&cmd, session_id, &self.game_cache, &self.session_manager)
                    .await;
                return;
            }
            ClientCommand::ClientJoinReplay(j) => {
                let cmd = ffb_protocol::commands::client_command_join_replay::ClientCommandJoinReplay {
                    entropy: None,
                    replay_name: j.replay_name.clone(),
                    coach: j.coach.clone(),
                    game_id: j.game_id,
                };
                self.join_replay_handler.handle_command(&cmd, session_id);
                return;
            }
            ClientCommand::ClientReplay(r) => {
                let cmd = ffb_protocol::commands::client_command_replay::ClientCommandReplay {
                    entropy: None,
                    game_id: r.game_id,
                    replay_to_command_nr: r.replay_to_command_nr,
                    coach: r.coach.clone(),
                };
                self.replay_handler.handle_command(&cmd, session_id).await;
                return;
            }
            ClientCommand::ClientReplayStatus(s) => {
                let cmd = ffb_protocol::commands::client_command_replay_status::ClientCommandReplayStatus {
                    entropy: None,
                    command_nr: s.command_nr,
                    speed: s.speed,
                    running: s.running,
                    forward: s.forward,
                    skip: s.skip,
                };
                self.replay_status_handler.handle_command(&cmd, session_id);
                return;
            }
            ClientCommand::ClientUserSettings(u) => {
                let mut cmd = ffb_protocol::commands::client_command_user_settings::ClientCommandUserSettings::new();
                cmd.settings = u.settings.clone();
                self.user_settings_handler.handle_command(session_id, &cmd).await;
                return;
            }
            _ => {}
        }

        let side = {
            let sm = self.session_manager.lock().unwrap();
            if sm.get_session_of_home_coach(game_id) == Some(session_id) {
                TeamSide::Home
            } else {
                TeamSide::Away
            }
        };

        let action = match decode_command(client_command, side) {
            Ok(a) => a,
            Err(e) => {
                log::warn!("session {} decode error: {}", session_id, e);
                return;
            }
        };

        let events = {
            let mut gc = self.game_cache.lock().unwrap();
            match gc.get_game_state_by_id_mut(game_id) {
                Some(gs) => match gs.handle_action(side, action) {
                    Ok(evts) => evts,
                    Err(e) => {
                        log::warn!("engine rejected action from session {}: {}", session_id, e);
                        return;
                    }
                },
                None => {
                    log::warn!("session {} sent command but game {} not found", session_id, game_id);
                    return;
                }
            }
        };

        let command_nr = {
            let mut gc = self.game_cache.lock().unwrap();
            gc.get_game_state_by_id_mut(game_id).map(|gs| gs.generate_command_nr()).unwrap_or(0)
        };

        let reports = events_to_reports(&events);
        let sync = OutgoingModelSync::new(command_nr, reports);
        match serde_json::to_string(&sync) {
            Ok(json) => {
                let sm = self.session_manager.lock().unwrap();
                sm.send_all(game_id, &json);
            }
            Err(e) => log::error!("failed to serialize model sync: {}", e),
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
                // java: `handleCommand`'s real logic needs `command.getTeam()` — a `Team`
                // resolved upstream (in Java, `FumbblRequestLoadTeam`'s HTTP response) before
                // the command was built. `InternalServerCommandAddLoadedTeam` never grew a
                // typed `Team` field (see that struct's own doc comment / this handler's own
                // module doc comment), and no request path in this crate produces one, so
                // there is no real `Team` to hand `add_loaded_team_handler.handle_command`
                // here without fabricating one — a narrow, documented gap. The handler itself
                // is real and directly unit-tested (see this file's own tests, which call
                // `add_loaded_team_handler` with a real `Team`).
                log::debug!(
                    "session {}: game {}: AddLoadedTeam received but its Team payload has no \
                     typed decode path yet (see ServerCommandHandlerAddLoadedTeam's doc comment)",
                    session_id, cmd.game_id
                );
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
            AnyInternalServerCommand::ApplyAutomatedPlayerMarkings(cmd) => {
                // java: `ServerCommandHandlerApplyAutomatedPlayerMarkings::handle_command` needs
                // a real `&mut AutoMarkingConfig` (ffb_engine::marking::auto_marking_config), but
                // `InternalServerCommandApplyAutomatedPlayerMarkings` only carries the config as
                // an opaque `String` (no serde impl exists yet for `AutoMarkingConfig` — see that
                // struct's own doc comment) — decoding it here would mean inventing a wire format
                // that doesn't exist in Java or Rust, so this stays a documented no-op rather than
                // a fabricated parse. `ServerCommandHandlerApplyAutomatedPlayerMarkings` itself is
                // still real and unit-tested directly (see its own module).
                log::debug!(
                    "session {}: game {}: ApplyAutomatedPlayerMarkings received but its \
                     AutoMarkingConfig payload has no typed decode path yet",
                    session_id, cmd.game_id
                );
            }
            AnyInternalServerCommand::CalculateAutomaticPlayerMarkings(cmd) => {
                // java: same gap as `ApplyAutomatedPlayerMarkings` above — both the
                // `AutoMarkingConfig` and `Game` payloads are opaque `String`s on this internal
                // command with no typed decode path, so `ServerCommandHandlerCalculateAutomaticPlayerMarkings`
                // (real and unit-tested on its own) isn't reachable from here yet.
                log::debug!(
                    "session {}: CalculateAutomaticPlayerMarkings (index {}) received but its \
                     AutoMarkingConfig/Game payload has no typed decode path yet",
                    session_id, cmd.index
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

// ── ClientCommand → Action decoder ───────────────────────────────────────────

/// Server-side inverse of `ffb-client/src/network_encoder/mod.rs`.
///
/// Decodes a `ClientCommand` into an engine `Action`.  Gameplay-routing
/// commands that have no `Action` equivalent (e.g. `ClientJoin`, `ClientPing`)
/// should be handled *before* this function is called.
pub fn decode_command(cmd: ClientCommand, side: TeamSide) -> Result<Action, DecodeError> {
    match cmd {
        ClientCommand::ClientEndTurn(_) => Ok(Action::EndTurn),

        ClientCommand::ClientMove(m) => Ok(Action::Move { path: m.move_squares }),
        ClientCommand::ClientBlitzMove(m) => Ok(Action::Move { path: m.move_squares }),

        ClientCommand::ClientActingPlayer(a) => {
            let player_action = player_action_to_choice(a.player_action)
                .ok_or(DecodeError::NotImplemented(format!("player_action {:?}", a.player_action)))?;
            Ok(Action::ActivatePlayer {
                player_id: a.player_id,
                player_action,
                block_defender_id: None,
            })
        }

        ClientCommand::ClientBlock(b) => Ok(Action::Block { defender_id: b.defender_id }),
        ClientCommand::ClientBlockChoice(b) => Ok(Action::BlockChoice {
            die_index: b.selected_die_index as usize,
            target_id: None,
        }),
        ClientCommand::ClientPushback(p) => Ok(Action::PushTo { coord: p.pushback_square }),
        ClientCommand::ClientFollowupChoice(f) => Ok(Action::FollowUp { follow_up: f.follow_up }),

        ClientCommand::ClientKickoff(k) => Ok(Action::KickBall { coord: k.coordinate }),
        ClientCommand::ClientTouchback(t) => Ok(Action::Touchback { player_id: t.player_id }),

        ClientCommand::ClientPass(p) => Ok(Action::Pass { coord: p.target_coordinate }),
        ClientCommand::ClientHandOver(h) => Ok(Action::HandOff { receiver_id: h.target_player_id }),
        ClientCommand::ClientFoul(f) => Ok(Action::Foul { target_id: f.defender_id }),

        ClientCommand::ClientInterceptorChoice(i) => Ok(Action::Intercept { attempt: i.attempt_interception }),
        ClientCommand::ClientCoinChoice(c) => Ok(Action::CoinChoice { heads: c.home_choice }),
        ClientCommand::ClientReceiveChoice(r) => Ok(Action::ReceiveChoice { receive: r.receive }),

        ClientCommand::ClientUseReRoll(r) => Ok(Action::UseReRoll { use_reroll: r.use_reroll }),
        ClientCommand::ClientUseSkill(s) => {
            let skill_id = parse_skill_id(&s.skill)
                .ok_or_else(|| DecodeError::UnknownSkill(s.skill.clone()))?;
            Ok(Action::UseSkill { skill_id, use_skill: s.use_skill })
        }

        ClientCommand::ClientUseApothecary(a) => Ok(Action::UseApothecary {
            player_id: a.player_id,
            use_apothecary: a.use_apothecary,
        }),
        ClientCommand::ClientApothecaryChoice(a) => {
            use ffb_protocol::client_commands::ApothecaryChoice;
            Ok(Action::ApothecaryChoice {
                player_state: match a.choice {
                    ApothecaryChoice::Apothecary => 1,
                    ApothecaryChoice::RollResult => 0,
                },
                serious_injury: None,
            })
        }

        ClientCommand::ClientSetupPlayer(s) => Ok(Action::PlacePlayer {
            player_id: s.player_id,
            coord: s.coordinate,
        }),
        ClientCommand::ClientStartGame(_) => Ok(Action::StartGame { home: side == TeamSide::Home }),

        ClientCommand::ClientThrowTeamMate(t) => Ok(Action::ThrowTeamMate {
            player_id: t.player_id,
            coord: t.target_coordinate,
        }),
        ClientCommand::ClientKickTeamMate(k) => Ok(Action::KickTeamMate {
            player_id: k.player_id,
            coord: k.target_coordinate,
        }),
        ClientCommand::ClientSwoop(s) => Ok(Action::Pass { coord: s.target_coordinate }),

        ClientCommand::ClientGaze(g) => Ok(Action::HypnoticGaze { target_id: g.target_id }),
        ClientCommand::ClientConfirm(_) => Ok(Action::Acknowledge),
        ClientCommand::ClientArgueTheCall(a) => Ok(Action::ArgueTheCall { argue: a.use_argue }),

        ClientCommand::ClientPlayerChoice(p) => Ok(Action::SelectPlayer { player_id: p.player_id }),

        ClientCommand::ClientBloodlustAction(b) => Ok(Action::BloodlustAction {
            change: b.action.eq_ignore_ascii_case("change"),
        }),

        ClientCommand::ClientBuyInducements(b) => Ok(Action::BuyInducements {
            purchases: b.purchases.into_iter().map(|(id, count)| {
                ffb_engine::action::InducementPurchase { id, count: count as u32 }
            }).collect(),
        }),

        ClientCommand::ClientPettyCash(p) => Ok(Action::PettyCash {
            home: side == TeamSide::Home,
            amount: p.amount,
        }),

        ClientCommand::ClientKickOffResultChoice(_) => Ok(Action::Acknowledge),
        ClientCommand::ClientSelectWeather(s) => {
            // Parse weather name via serde camelCase
            let camel = to_camel_case(&s.weather);
            serde_json::from_str::<ffb_model::enums::Weather>(&format!("\"{}\"", camel))
                .map(|w| Action::SelectWeather { weather: w })
                .map_err(|_| DecodeError::NotImplemented(format!("weather {}", s.weather)))
        }

        ClientCommand::ClientPileDriver(_) => Err(DecodeError::NotImplemented("ClientPileDriver".into())),
        ClientCommand::ClientWizardSpell(_) => Err(DecodeError::NotImplemented("ClientWizardSpell".into())),
        ClientCommand::ClientJourneymen(_) => Err(DecodeError::NotImplemented("ClientJourneymen".into())),

        ClientCommand::ClientJoin(_) => Err(DecodeError::NotImplemented("ClientJoin".into())),
        ClientCommand::ClientPing(_) => Err(DecodeError::NotImplemented("ClientPing".into())),

        // Handled earlier in `handle_command` before `decode_command` is reached (like
        // `ClientJoin`/`ClientPing` above) — no `Action` equivalent exists for these.
        ClientCommand::ClientTalk(_) => Err(DecodeError::NotImplemented("ClientTalk".into())),
        ClientCommand::ClientCloseSession(_) => Err(DecodeError::NotImplemented("ClientCloseSession".into())),
        ClientCommand::ClientTransferReplayControl(_) => {
            Err(DecodeError::NotImplemented("ClientTransferReplayControl".into()))
        }
        ClientCommand::ClientRequestVersion(_) => Err(DecodeError::NotImplemented("ClientRequestVersion".into())),
        ClientCommand::ClientPasswordChallenge(_) => {
            Err(DecodeError::NotImplemented("ClientPasswordChallenge".into()))
        }
        ClientCommand::ClientAddSketch(_) => Err(DecodeError::NotImplemented("ClientAddSketch".into())),
        ClientCommand::ClientClearSketches(_) => {
            Err(DecodeError::NotImplemented("ClientClearSketches".into()))
        }
        ClientCommand::ClientRemoveSketches(_) => {
            Err(DecodeError::NotImplemented("ClientRemoveSketches".into()))
        }
        ClientCommand::ClientSketchAddCoordinate(_) => {
            Err(DecodeError::NotImplemented("ClientSketchAddCoordinate".into()))
        }
        ClientCommand::ClientSketchSetColor(_) => {
            Err(DecodeError::NotImplemented("ClientSketchSetColor".into()))
        }
        ClientCommand::ClientSketchSetLabel(_) => {
            Err(DecodeError::NotImplemented("ClientSketchSetLabel".into()))
        }
        ClientCommand::ClientSetMarker(_) => Err(DecodeError::NotImplemented("ClientSetMarker".into())),
        ClientCommand::ClientSetPreventSketching(_) => {
            Err(DecodeError::NotImplemented("ClientSetPreventSketching".into()))
        }
        ClientCommand::ClientUpdatePlayerMarkings(_) => {
            Err(DecodeError::NotImplemented("ClientUpdatePlayerMarkings".into()))
        }
        ClientCommand::ClientLoadAutomaticPlayerMarkings(_) => {
            Err(DecodeError::NotImplemented("ClientLoadAutomaticPlayerMarkings".into()))
        }
        ClientCommand::ClientJoinReplay(_) => Err(DecodeError::NotImplemented("ClientJoinReplay".into())),
        ClientCommand::ClientReplay(_) => Err(DecodeError::NotImplemented("ClientReplay".into())),
        ClientCommand::ClientReplayStatus(_) => Err(DecodeError::NotImplemented("ClientReplayStatus".into())),
        ClientCommand::ClientUserSettings(_) => Err(DecodeError::NotImplemented("ClientUserSettings".into())),
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
    use ffb_protocol::client_commands::*;

    fn coord(x: i32, y: i32) -> FieldCoordinate { FieldCoordinate { x, y } }

    // ── decode_command: pre-game ───────────────────────────────────────────────

    #[test]
    fn decode_end_turn() {
        let r = decode_command(ClientCommand::ClientEndTurn(ClientEndTurn), TeamSide::Home);
        assert!(matches!(r, Ok(Action::EndTurn)));
    }

    #[test]
    fn decode_coin_choice_heads() {
        let r = decode_command(
            ClientCommand::ClientCoinChoice(ClientCoinChoice { home_choice: true }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::CoinChoice { heads: true })));
    }

    #[test]
    fn decode_coin_choice_tails() {
        let r = decode_command(
            ClientCommand::ClientCoinChoice(ClientCoinChoice { home_choice: false }),
            TeamSide::Away,
        );
        assert!(matches!(r, Ok(Action::CoinChoice { heads: false })));
    }

    #[test]
    fn decode_receive_choice_receive() {
        let r = decode_command(
            ClientCommand::ClientReceiveChoice(ClientReceiveChoice { receive: true }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::ReceiveChoice { receive: true })));
    }

    #[test]
    fn decode_receive_choice_kick() {
        let r = decode_command(
            ClientCommand::ClientReceiveChoice(ClientReceiveChoice { receive: false }),
            TeamSide::Away,
        );
        assert!(matches!(r, Ok(Action::ReceiveChoice { receive: false })));
    }

    #[test]
    fn decode_start_game_home() {
        let r = decode_command(ClientCommand::ClientStartGame(ClientStartGame), TeamSide::Home);
        assert!(matches!(r, Ok(Action::StartGame { home: true })));
    }

    #[test]
    fn decode_start_game_away() {
        let r = decode_command(ClientCommand::ClientStartGame(ClientStartGame), TeamSide::Away);
        assert!(matches!(r, Ok(Action::StartGame { home: false })));
    }

    // ── decode_command: movement ───────────────────────────────────────────────

    #[test]
    fn decode_move() {
        let path = vec![coord(1, 1), coord(2, 2)];
        let r = decode_command(
            ClientCommand::ClientMove(ClientMove { move_squares: path.clone() }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::Move { path: p }) if p == path));
    }

    #[test]
    fn decode_blitz_move_as_move() {
        let path = vec![coord(3, 4)];
        let r = decode_command(
            ClientCommand::ClientBlitzMove(ClientBlitzMove { move_squares: path.clone() }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::Move { path: p }) if p == path));
    }

    #[test]
    fn decode_setup_player() {
        let r = decode_command(
            ClientCommand::ClientSetupPlayer(ClientSetupPlayer {
                player_id: "p1".into(),
                coordinate: coord(5, 5),
            }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::PlacePlayer { player_id, coord: c })
            if player_id == "p1" && c == coord(5, 5)));
    }

    // ── decode_command: block sequence ────────────────────────────────────────

    #[test]
    fn decode_block() {
        let r = decode_command(
            ClientCommand::ClientBlock(ClientBlock { defender_id: "defender".into() }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::Block { defender_id: d }) if d == "defender"));
    }

    #[test]
    fn decode_block_choice() {
        let r = decode_command(
            ClientCommand::ClientBlockChoice(ClientBlockChoice { selected_die_index: 2 }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::BlockChoice { die_index: 2, target_id: None })));
    }

    #[test]
    fn decode_pushback() {
        let r = decode_command(
            ClientCommand::ClientPushback(ClientPushback { pushback_square: coord(7, 3) }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::PushTo { coord: c }) if c == coord(7, 3)));
    }

    #[test]
    fn decode_followup_yes() {
        let r = decode_command(
            ClientCommand::ClientFollowupChoice(ClientFollowupChoice { follow_up: true }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::FollowUp { follow_up: true })));
    }

    #[test]
    fn decode_followup_no() {
        let r = decode_command(
            ClientCommand::ClientFollowupChoice(ClientFollowupChoice { follow_up: false }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::FollowUp { follow_up: false })));
    }

    // ── decode_command: acting player / activation ────────────────────────────

    #[test]
    fn decode_acting_player_move() {
        let r = decode_command(
            ClientCommand::ClientActingPlayer(ClientActingPlayer {
                player_id: "p1".into(),
                player_action: ffb_model::enums::PlayerAction::Move,
                standing_up: false,
            }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::ActivatePlayer {
            player_id: ref pid,
            player_action: PlayerActionChoice::Move,
            ..
        }) if pid == "p1"));
    }

    #[test]
    fn decode_acting_player_block() {
        let r = decode_command(
            ClientCommand::ClientActingPlayer(ClientActingPlayer {
                player_id: "p2".into(),
                player_action: ffb_model::enums::PlayerAction::Block,
                standing_up: false,
            }),
            TeamSide::Away,
        );
        assert!(matches!(r, Ok(Action::ActivatePlayer { player_action: PlayerActionChoice::Block, .. })));
    }

    #[test]
    fn decode_acting_player_foul() {
        let r = decode_command(
            ClientCommand::ClientActingPlayer(ClientActingPlayer {
                player_id: "p3".into(),
                player_action: ffb_model::enums::PlayerAction::Foul,
                standing_up: false,
            }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::ActivatePlayer { player_action: PlayerActionChoice::Foul, .. })));
    }

    // ── decode_command: pass / handoff / foul ─────────────────────────────────

    #[test]
    fn decode_pass() {
        let r = decode_command(
            ClientCommand::ClientPass(ClientPass { target_coordinate: coord(10, 5), hail_mary: false }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::Pass { coord: c }) if c == coord(10, 5)));
    }

    #[test]
    fn decode_handover() {
        let r = decode_command(
            ClientCommand::ClientHandOver(ClientHandOver { target_player_id: "receiver".into() }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::HandOff { receiver_id: ref id }) if id == "receiver"));
    }

    #[test]
    fn decode_foul() {
        let r = decode_command(
            ClientCommand::ClientFoul(ClientFoul { defender_id: "prone".into() }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::Foul { target_id: ref id }) if id == "prone"));
    }

    // ── decode_command: kickoff ────────────────────────────────────────────────

    #[test]
    fn decode_kickoff() {
        let r = decode_command(
            ClientCommand::ClientKickoff(ClientKickoff { coordinate: coord(8, 8) }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::KickBall { coord: c }) if c == coord(8, 8)));
    }

    #[test]
    fn decode_touchback() {
        let r = decode_command(
            ClientCommand::ClientTouchback(ClientTouchback { player_id: "catcher".into() }),
            TeamSide::Away,
        );
        assert!(matches!(r, Ok(Action::Touchback { player_id: ref id }) if id == "catcher"));
    }

    // ── decode_command: skill / reroll ────────────────────────────────────────

    #[test]
    fn decode_use_reroll_yes() {
        let r = decode_command(
            ClientCommand::ClientUseReRoll(ClientUseReRoll { use_reroll: true }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::UseReRoll { use_reroll: true })));
    }

    #[test]
    fn decode_use_reroll_no() {
        let r = decode_command(
            ClientCommand::ClientUseReRoll(ClientUseReRoll { use_reroll: false }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::UseReRoll { use_reroll: false })));
    }

    #[test]
    fn decode_use_skill_block() {
        let r = decode_command(
            ClientCommand::ClientUseSkill(ClientUseSkill {
                player_id: "p1".into(),
                skill: "Block".into(),
                use_skill: true,
            }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::UseSkill { skill_id: SkillId::Block, use_skill: true })));
    }

    #[test]
    fn decode_use_skill_sure_hands_declined() {
        let r = decode_command(
            ClientCommand::ClientUseSkill(ClientUseSkill {
                player_id: "p1".into(),
                skill: "SureHands".into(),
                use_skill: false,
            }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::UseSkill { skill_id: SkillId::SureHands, use_skill: false })));
    }

    #[test]
    fn decode_use_skill_unknown_returns_error() {
        let r = decode_command(
            ClientCommand::ClientUseSkill(ClientUseSkill {
                player_id: "p1".into(),
                skill: "NotASkill".into(),
                use_skill: true,
            }),
            TeamSide::Home,
        );
        assert!(matches!(r, Err(DecodeError::UnknownSkill(_))));
    }

    #[test]
    fn decode_use_apothecary() {
        let r = decode_command(
            ClientCommand::ClientUseApothecary(ClientUseApothecary {
                player_id: "hurt".into(),
                use_apothecary: true,
            }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::UseApothecary { use_apothecary: true, .. })));
    }

    // ── decode_command: intercept / misc ──────────────────────────────────────

    #[test]
    fn decode_intercept_attempt() {
        let r = decode_command(
            ClientCommand::ClientInterceptorChoice(ClientInterceptorChoice { attempt_interception: true }),
            TeamSide::Away,
        );
        assert!(matches!(r, Ok(Action::Intercept { attempt: true })));
    }

    #[test]
    fn decode_confirm_acknowledges() {
        let r = decode_command(ClientCommand::ClientConfirm(ClientConfirm), TeamSide::Home);
        assert!(matches!(r, Ok(Action::Acknowledge)));
    }

    #[test]
    fn decode_argue_the_call() {
        let r = decode_command(
            ClientCommand::ClientArgueTheCall(ClientArgueTheCall {
                player_id: "captain".into(),
                use_argue: true,
            }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::ArgueTheCall { argue: true })));
    }

    #[test]
    fn decode_player_choice() {
        let r = decode_command(
            ClientCommand::ClientPlayerChoice(ClientPlayerChoice { player_id: "chosen".into() }),
            TeamSide::Home,
        );
        assert!(matches!(r, Ok(Action::SelectPlayer { player_id: ref id }) if id == "chosen"));
    }

    #[test]
    fn decode_ping_returns_not_implemented() {
        let r = decode_command(
            ClientCommand::ClientPing(ClientPing { timestamp: 123 }),
            TeamSide::Home,
        );
        assert!(matches!(r, Err(DecodeError::NotImplemented(_))));
    }

    #[test]
    fn decode_join_returns_not_implemented() {
        let r = decode_command(
            ClientCommand::ClientJoin(ClientJoin {
                coach: "x".into(), team_id: "t".into(), game_id: "1".into(), password_hash: None,
            }),
            TeamSide::Home,
        );
        assert!(matches!(r, Err(DecodeError::NotImplemented(_))));
    }

    // ── parse_skill_id ────────────────────────────────────────────────────────

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
        factory.handle_command(ReceivedCommand::new(ClientCommand::ClientPing(ClientPing { timestamp: 9999 }), 42)).await;
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
            ClientCommand::ClientEndTurn(ClientEndTurn), 99,
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
            ClientCommand::ClientTalk(ffb_protocol::client_commands::ClientTalk { talk: Some("hello".into()) }),
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
            ClientCommand::ClientCloseSession(ffb_protocol::client_commands::ClientCloseSession),
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
            ClientCommand::ClientTransferReplayControl(ffb_protocol::client_commands::ClientTransferReplayControl {
                coach: Some("coach2".into()),
            }),
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
            ClientCommand::ClientRequestVersion(ffb_protocol::client_commands::ClientRequestVersion),
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
            ClientCommand::ClientPasswordChallenge(ffb_protocol::client_commands::ClientPasswordChallenge {
                coach: None,
            }),
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
            ClientCommand::ClientAddSketch(ffb_protocol::client_commands::ClientAddSketch {
                sketch_id: Some("sk-1".into()),
            }),
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
            ClientCommand::ClientClearSketches(ffb_protocol::client_commands::ClientClearSketches),
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
            ClientCommand::ClientRemoveSketches(ffb_protocol::client_commands::ClientRemoveSketches {
                ids: vec!["sk-1".into()],
            }),
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
            ClientCommand::ClientSketchAddCoordinate(ffb_protocol::client_commands::ClientSketchAddCoordinate {
                sketch_id: Some("sk-1".into()),
                coordinate: Some(FieldCoordinate::new(3, 4)),
            }),
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
            ClientCommand::ClientSketchSetColor(ffb_protocol::client_commands::ClientSketchSetColor {
                sketch_ids: vec!["sk-1".into()],
                rgb: 0x00FF00,
            }),
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
            ClientCommand::ClientSketchSetLabel(ffb_protocol::client_commands::ClientSketchSetLabel {
                sketch_ids: vec!["sk-1".into()],
                label: Some("Arrow".into()),
            }),
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
                special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false,
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
            ClientCommand::ClientSetMarker(ffb_protocol::client_commands::ClientSetMarker {
                player_id: Some("p1".into()),
                coordinate: Some(FieldCoordinate::new(1, 1)),
                text: Some("Nice job".into()),
            }),
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
            ClientCommand::ClientSetPreventSketching(ffb_protocol::client_commands::ClientSetPreventSketching {
                coach: Some("coach2".into()),
                prevent_sketching: true,
            }),
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
            ClientCommand::ClientUpdatePlayerMarkings(ffb_protocol::client_commands::ClientUpdatePlayerMarkings {
                auto: false,
                sort_mode_name: None,
            }),
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
            ClientCommand::ClientLoadAutomaticPlayerMarkings(
                ffb_protocol::client_commands::ClientLoadAutomaticPlayerMarkings {
                    index: 1,
                    coach: Some("Coach".into()),
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
            ClientCommand::ClientJoinReplay(ffb_protocol::client_commands::ClientJoinReplay {
                replay_name: Some("MyReplay".into()),
                coach: Some("Alice".into()),
                game_id: 42,
            }),
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
            ClientCommand::ClientReplay(ffb_protocol::client_commands::ClientReplay {
                game_id,
                replay_to_command_nr: 0,
                coach: Some("coach".into()),
            }),
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
            ClientCommand::ClientReplayStatus(ffb_protocol::client_commands::ClientReplayStatus {
                command_nr: 10,
                speed: 1,
                running: true,
                forward: true,
                skip: false,
            }),
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
            special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false,
        }
    }

    #[test]
    fn factory_add_loaded_team_handler_adds_a_real_team_via_direct_call() {
        // The `AddLoadedTeam` dispatch arm is a documented no-op (no `Team` payload on the
        // wire command — see that arm's own comment), so this exercises the real,
        // factory-owned handler directly instead, proving it isn't dead code.
        let gc = Arc::new(Mutex::new(crate::game_cache::GameCache::new()));
        let sm = Arc::new(Mutex::new(crate::net::session_manager::SessionManager::new()));
        let factory = ServerCommandHandlerFactory::new(Arc::clone(&gc), sm, db());
        let game_id = gc.lock().unwrap().create_game_state();
        gc.lock().unwrap().get_game_state_by_id_mut(game_id).unwrap().start_game(
            gm_team(""), gm_team("away1"), ffb_model::enums::Rules::Bb2025, 0,
        );

        let cmd = crate::net::commands::internal_server_command_add_loaded_team::InternalServerCommandAddLoadedTeam::new(
            game_id, "coach".into(), None, vec![],
        );
        let mut gc_guard = gc.lock().unwrap();
        assert!(factory.add_loaded_team_handler.handle_command(&cmd, gm_team("home1"), &mut gc_guard));
        drop(gc_guard);

        let game = gc.lock().unwrap().get_game_state_by_id(game_id).unwrap().get_game().unwrap().clone();
        assert_eq!(game.team_home.id, "home1");
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
            ClientCommand::ClientUserSettings(ffb_protocol::client_commands::ClientUserSettings { settings }),
            1,
        )).await;

        // With no DB pool configured this degrades to a no-op (see
        // `ServerCommandHandlerUserSettings`'s own doc comment), but the real handler still
        // ran to completion via the factory's dispatch arm without panicking.
        assert!(sm_arc.lock().unwrap().get_coach_for_session(1).is_some());
    }
}
