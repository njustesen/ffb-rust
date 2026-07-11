/// 1:1 translation of com.fumbbl.ffb.server.request.ServerRequestLoadReplay.
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use ffb_model::enums::GameStatus;

use crate::game_cache::GameCache;
use crate::game_state::GameState;
use crate::model::received_command::{ReceivedCommand, SessionId};
use crate::net::commands::any_internal_server_command::AnyInternalServerCommand;
use crate::net::commands::internal_server_command_delete_game::InternalServerCommandDeleteGame;
use crate::net::commands::internal_server_command_replay_loaded::InternalServerCommandReplayLoaded;
use crate::net::commands::internal_server_command_upload_game::InternalServerCommandUploadGame;
use crate::request::fumbbl::util_fumbbl_request::HttpClient;
use crate::request::server_request::ServerRequest;

pub struct ServerRequestLoadReplay {
    pub game_id: i64,
    pub replay_to_command_nr: i32,
    pub mode: i32,
    pub team_id: String,
    pub coach: String,
    request_url: String,
}

impl ServerRequestLoadReplay {
    pub const LOAD_GAME: i32 = 1;
    pub const DELETE_GAME: i32 = 2;
    pub const UPLOAD_GAME: i32 = 3;

    pub fn new(
        game_id: i64,
        replay_to_command_nr: i32,
        mode: i32,
        team_id: String,
        coach: String,
    ) -> Self {
        Self {
            game_id,
            replay_to_command_nr,
            mode,
            team_id,
            coach,
            request_url: String::new(),
        }
    }

    pub fn get_game_id(&self) -> i64 {
        self.game_id
    }

    pub fn get_replay_to_command_nr(&self) -> i32 {
        self.replay_to_command_nr
    }

    pub fn get_mode(&self) -> i32 {
        self.mode
    }

    pub fn get_request_url(&self) -> &str {
        &self.request_url
    }

    pub fn set_request_url(&mut self, url: String) {
        self.request_url = url;
    }

    /// Builds the BACKUP_URL_LOAD URL for [`Self::game_id`], fetches it, and returns the raw
    /// JSON payload if present. Java parses this into a `GameState` and, depending on
    /// [`Self::mode`], dispatches `InternalServerCommandReplayLoaded`,
    /// `InternalServerCommandDeleteGame`, or `InternalServerCommandUploadGame`; that
    /// session/command dispatch has no equivalent yet in this simplified server crate, so the
    /// caller is expected to act on the returned JSON per [`Self::mode`].
    pub fn process(
        &mut self,
        client: &dyn super::fumbbl::util_fumbbl_request::HttpClient,
        load_url_template: &str,
    ) -> Result<Option<String>, String> {
        let url = super::fumbbl::util_fumbbl_request::UtilFumbblRequest::bind(
            load_url_template,
            &[self.game_id.to_string().as_str()],
        );
        self.set_request_url(url);
        let json_string = client.fetch_page(self.get_request_url())?;
        if json_string.is_empty() {
            return Ok(None);
        }
        Ok(Some(json_string))
    }
}

/// `ServerRequest` adapter around [`ServerRequestLoadReplay`] — this is the Rust shape of
/// Java's `ServerRequestLoadReplay.process(ServerRequestProcessor)`, the method that actually
/// implements `ServerRequest.process()` in the Java class hierarchy. The `ServerRequest` trait
/// here (`fn process(&self) -> Result<(), String>`) takes no parameters, so — following the
/// established pattern in `util/marker_loading_service.rs`'s `QueuedLoadPlayerMarkingsRequest`
/// and `handler/server_command_handler_update_player_markings.rs`'s same-named adapter — the
/// dependencies Java reaches via `pRequestProcessor.getServer().getX()` (the HTTP client, the
/// `GameCache`, and the dispatch queue that stands in for `server.getCommunication()`) are
/// threaded through explicitly as fields instead.
///
/// Ports all three of Java's `process()` mode branches (`LOAD_GAME`/`DELETE_GAME`/
/// `UPLOAD_GAME`) since they are one shared method in Java, not three; the `UPLOAD_GAME`
/// branch (this phase's actual target — `ServerCommandHandlerUploadGame`'s missing-game
/// path) is exercised by this file's own tests. Java's `LOAD_GAME`-not-found branch
/// (`communication.sendStatus(session, ServerStatus.REPLAY_UNAVAILABLE, "")`) has no
/// `SessionManager`/status-send dependency threaded into this adapter (out of scope for the
/// `UPLOAD_GAME` branch this phase targets), so that one sub-case is a no-op here rather than
/// sending the status message — a narrower, documented gap.
pub struct QueuedServerRequestLoadReplay {
    request: Mutex<ServerRequestLoadReplay>,
    client: Arc<dyn HttpClient + Send + Sync>,
    load_url_template: String,
    game_cache: Arc<Mutex<GameCache>>,
    dispatch_tx: mpsc::UnboundedSender<ReceivedCommand>,
    session_id: SessionId,
}

impl QueuedServerRequestLoadReplay {
    pub fn new(
        request: ServerRequestLoadReplay,
        client: Arc<dyn HttpClient + Send + Sync>,
        load_url_template: impl Into<String>,
        game_cache: Arc<Mutex<GameCache>>,
        dispatch_tx: mpsc::UnboundedSender<ReceivedCommand>,
        session_id: SessionId,
    ) -> Self {
        Self {
            request: Mutex::new(request),
            client,
            load_url_template: load_url_template.into(),
            game_cache,
            dispatch_tx,
            session_id,
        }
    }
}

impl ServerRequest for QueuedServerRequestLoadReplay {
    /// Java: `ServerRequestLoadReplay.process(ServerRequestProcessor)`.
    fn process(&self) -> Result<(), String> {
        let (game_id, mode, replay_to_command_nr, team_id, coach, json_string) = {
            let mut request = self.request.lock().unwrap();
            let json_string = match request.process(self.client.as_ref(), &self.load_url_template) {
                Ok(json) => json,
                // Java: `catch (Exception parseException) { ...; return; }`.
                Err(_) => return Ok(()),
            };
            (
                request.get_game_id(),
                request.get_mode(),
                request.get_replay_to_command_nr(),
                request.team_id.clone(),
                request.coach.clone(),
                json_string,
            )
        };

        // Java: `gameState = new GameState(server); gameState.initFrom(...)` — wrapped in a
        // try/catch that logs and returns on any parse failure.
        let game_state = match json_string {
            Some(json) => {
                let mut gs = GameState::new(game_id);
                match gs.init_from(&json) {
                    Ok(()) => Some(gs),
                    Err(_) => return Ok(()),
                }
            }
            None => None,
        };

        if mode == ServerRequestLoadReplay::LOAD_GAME {
            if let Some(mut gs) = game_state {
                if let Some(game) = gs.get_game_mut() {
                    game.status = GameStatus::Loading;
                }
                self.game_cache.lock().unwrap().add_game(gs);
                let replay_loaded = InternalServerCommandReplayLoaded::new(game_id, replay_to_command_nr, coach);
                let _ = self.dispatch_tx.send(ReceivedCommand::new_internal(
                    AnyInternalServerCommand::ReplayLoaded(replay_loaded),
                    self.session_id,
                ));
            }
            // else: Java sends ServerStatus.REPLAY_UNAVAILABLE — see this struct's doc comment.
            return Ok(());
        }

        if mode == ServerRequestLoadReplay::DELETE_GAME {
            if game_state.is_some() {
                let delete_game = InternalServerCommandDeleteGame::new(game_id, false);
                let _ = self.dispatch_tx.send(ReceivedCommand::new_internal(
                    AnyInternalServerCommand::DeleteGame(delete_game),
                    self.session_id,
                ));
            }
            return Ok(());
        }

        if mode == ServerRequestLoadReplay::UPLOAD_GAME {
            if let Some(mut gs) = game_state {
                if let Some(game) = gs.get_game_mut() {
                    game.status = GameStatus::Loading;
                }
                self.game_cache.lock().unwrap().add_game(gs);
                let conceding_team_id = if team_id.is_empty() { None } else { Some(team_id) };
                let upload_command = InternalServerCommandUploadGame::new_with_conceding(game_id, conceding_team_id);
                let _ = self.dispatch_tx.send(ReceivedCommand::new_internal(
                    AnyInternalServerCommand::UploadGame(upload_command),
                    self.session_id,
                ));
            }
        }

        Ok(())
    }

    fn get_request_url(&self) -> &str {
        // Locking to read a `&str` behind a `Mutex` isn't expressible without leaking — same
        // best-effort behavior as the other `ServerRequest` adapters in this crate (see
        // `util/marker_loading_service.rs`'s `QueuedLoadPlayerMarkingsRequest`).
        ""
    }

    fn set_request_url(&mut self, _url: String) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::commands::internal_server_command::InternalServerCommand;
    use crate::request::fumbbl::util_fumbbl_request::MockHttpClient;

    #[test]
    fn construct() {
        let r = ServerRequestLoadReplay::new(1, 0, ServerRequestLoadReplay::LOAD_GAME, String::new(), String::new());
        assert_eq!(r.get_game_id(), 1);
        assert_eq!(r.get_mode(), ServerRequestLoadReplay::LOAD_GAME);
    }

    #[test]
    fn process_builds_url_and_returns_json() {
        let client = MockHttpClient {
            response: Ok("{\"half\":1}".to_string()),
        };
        let mut r = ServerRequestLoadReplay::new(42, 0, ServerRequestLoadReplay::LOAD_GAME, String::new(), String::new());
        let json = r.process(&client, "http://backup/load/$1").unwrap();
        assert_eq!(r.get_request_url(), "http://backup/load/42");
        assert_eq!(json, Some("{\"half\":1}".to_string()));
    }

    #[test]
    fn process_empty_response_returns_none() {
        let client = MockHttpClient { response: Ok(String::new()) };
        let mut r = ServerRequestLoadReplay::new(42, 0, ServerRequestLoadReplay::DELETE_GAME, String::new(), String::new());
        let json = r.process(&client, "http://backup/load/$1").unwrap();
        assert!(json.is_none());
    }

    // ── QueuedServerRequestLoadReplay ────────────────────────────────────────────

    fn team(id: &str) -> ffb_model::model::team::Team {
        ffb_model::model::team::Team {
            id: id.into(),
            name: id.into(),
            race: "Human".into(),
            roster_id: "human".into(),
            coach: "coach".into(),
            rerolls: 0,
            apothecaries: 0,
            bribes: 0,
            master_chefs: 0,
            prayers_to_nuffle: 0,
            bloodweiser_kegs: 0,
            riotous_rookies: 0,
            cheerleaders: 0,
            assistant_coaches: 0,
            fan_factor: 0,
            dedicated_fans: 0,
            team_value: 0,
            treasury: 0,
            special_rules: vec![],
            players: vec![],
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn backup_game_json() -> String {
        let game = ffb_model::model::game::Game::new(team("home"), team("away"), ffb_model::enums::Rules::Bb2025);
        serde_json::to_string(&game).unwrap()
    }

    #[test]
    fn queued_upload_game_missing_backup_is_a_noop() {
        let client: Arc<dyn HttpClient + Send + Sync> = Arc::new(MockHttpClient { response: Ok(String::new()) });
        let game_cache = Arc::new(Mutex::new(GameCache::new()));
        let (dispatch_tx, mut dispatch_rx) = mpsc::unbounded_channel();
        let request = ServerRequestLoadReplay::new(42, 0, ServerRequestLoadReplay::UPLOAD_GAME, "teamA".into(), String::new());
        let queued = QueuedServerRequestLoadReplay::new(
            request, client, "http://backup/load/$1", Arc::clone(&game_cache), dispatch_tx, 7,
        );

        queued.process().unwrap();

        assert!(game_cache.lock().unwrap().get_game_state_by_id(42).is_none());
        assert!(dispatch_rx.try_recv().is_err());
    }

    #[test]
    fn queued_upload_game_rehydrates_cache_and_redispatches_upload_game() {
        let client: Arc<dyn HttpClient + Send + Sync> = Arc::new(MockHttpClient { response: Ok(backup_game_json()) });
        let game_cache = Arc::new(Mutex::new(GameCache::new()));
        let (dispatch_tx, mut dispatch_rx) = mpsc::unbounded_channel();
        let request = ServerRequestLoadReplay::new(42, 0, ServerRequestLoadReplay::UPLOAD_GAME, "teamA".into(), String::new());
        let queued = QueuedServerRequestLoadReplay::new(
            request, client, "http://backup/load/$1", Arc::clone(&game_cache), dispatch_tx, 7,
        );

        queued.process().unwrap();

        let cache = game_cache.lock().unwrap();
        let gs = cache.get_game_state_by_id(42).expect("game should be re-added to the cache");
        assert_eq!(gs.get_game().unwrap().status, GameStatus::Loading);
        drop(cache);

        let received = dispatch_rx.try_recv().expect("expected a redispatched UploadGame command");
        assert_eq!(received.session_id, 7);
        match received.command {
            crate::model::received_command::ReceivedNetCommand::Internal(AnyInternalServerCommand::UploadGame(cmd)) => {
                assert_eq!(cmd.game_id, 42);
                assert_eq!(cmd.get_conceding_team_id(), Some("teamA"));
            }
            _ => panic!("expected an internal UploadGame command"),
        }
    }

    #[test]
    fn queued_load_game_rehydrates_cache_and_redispatches_replay_loaded() {
        let client: Arc<dyn HttpClient + Send + Sync> = Arc::new(MockHttpClient { response: Ok(backup_game_json()) });
        let game_cache = Arc::new(Mutex::new(GameCache::new()));
        let (dispatch_tx, mut dispatch_rx) = mpsc::unbounded_channel();
        let request = ServerRequestLoadReplay::new(9, 3, ServerRequestLoadReplay::LOAD_GAME, String::new(), "Kalimar".into());
        let queued = QueuedServerRequestLoadReplay::new(
            request, client, "http://backup/load/$1", Arc::clone(&game_cache), dispatch_tx, 1,
        );

        queued.process().unwrap();

        assert!(game_cache.lock().unwrap().get_game_state_by_id(9).is_some());
        let received = dispatch_rx.try_recv().expect("expected a redispatched ReplayLoaded command");
        match received.command {
            crate::model::received_command::ReceivedNetCommand::Internal(AnyInternalServerCommand::ReplayLoaded(cmd)) => {
                assert_eq!(cmd.get_game_id(), 9);
                assert_eq!(cmd.get_replay_to_command_nr(), 3);
                assert_eq!(cmd.get_coach(), "Kalimar");
            }
            _ => panic!("expected an internal ReplayLoaded command"),
        }
    }

    #[test]
    fn queued_delete_game_rehydrates_and_redispatches_delete_game() {
        let client: Arc<dyn HttpClient + Send + Sync> = Arc::new(MockHttpClient { response: Ok(backup_game_json()) });
        let game_cache = Arc::new(Mutex::new(GameCache::new()));
        let (dispatch_tx, mut dispatch_rx) = mpsc::unbounded_channel();
        let request = ServerRequestLoadReplay::new(5, 0, ServerRequestLoadReplay::DELETE_GAME, String::new(), String::new());
        let queued = QueuedServerRequestLoadReplay::new(
            request, client, "http://backup/load/$1", Arc::clone(&game_cache), dispatch_tx, 2,
        );

        queued.process().unwrap();

        let received = dispatch_rx.try_recv().expect("expected a redispatched DeleteGame command");
        match received.command {
            crate::model::received_command::ReceivedNetCommand::Internal(AnyInternalServerCommand::DeleteGame(cmd)) => {
                assert_eq!(cmd.get_game_id(), 5);
            }
            _ => panic!("expected an internal DeleteGame command"),
        }
    }

    #[test]
    fn queued_invalid_json_is_a_noop() {
        let client: Arc<dyn HttpClient + Send + Sync> = Arc::new(MockHttpClient { response: Ok("not json".to_string()) });
        let game_cache = Arc::new(Mutex::new(GameCache::new()));
        let (dispatch_tx, mut dispatch_rx) = mpsc::unbounded_channel();
        let request = ServerRequestLoadReplay::new(1, 0, ServerRequestLoadReplay::UPLOAD_GAME, String::new(), String::new());
        let queued = QueuedServerRequestLoadReplay::new(
            request, client, "http://backup/load/$1", Arc::clone(&game_cache), dispatch_tx, 1,
        );

        queued.process().unwrap();

        assert!(game_cache.lock().unwrap().get_game_state_by_id(1).is_none());
        assert!(dispatch_rx.try_recv().is_err());
    }

    #[test]
    fn queued_http_error_is_a_noop() {
        let client: Arc<dyn HttpClient + Send + Sync> = Arc::new(MockHttpClient { response: Err("network down".to_string()) });
        let game_cache = Arc::new(Mutex::new(GameCache::new()));
        let (dispatch_tx, mut dispatch_rx) = mpsc::unbounded_channel();
        let request = ServerRequestLoadReplay::new(1, 0, ServerRequestLoadReplay::UPLOAD_GAME, String::new(), String::new());
        let queued = QueuedServerRequestLoadReplay::new(
            request, client, "http://backup/load/$1", Arc::clone(&game_cache), dispatch_tx, 1,
        );

        queued.process().unwrap();

        assert!(dispatch_rx.try_recv().is_err());
    }
}
