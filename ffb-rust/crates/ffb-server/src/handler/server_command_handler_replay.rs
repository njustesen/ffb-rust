/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerReplay.
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use ffb_engine::server_replayer::ServerReplayer;
use ffb_model::enums::NetCommandId;
use ffb_protocol::commands::client_command_replay::ClientCommandReplay;
use ffb_protocol::commands::server_command_game_state::ServerCommandGameState;
use crate::db::db_connection_manager::DbConnectionManager;
use crate::game_cache::GameCache;
use crate::model::received_command::{ReceivedCommand, SessionId};
use crate::net::session_manager::SessionManager;
use crate::request::fumbbl::util_fumbbl_request::HttpClient;
use crate::request::server_request_load_replay::{QueuedServerRequestLoadReplay, ServerRequestLoadReplay};
use crate::request::server_request_processor::ServerRequestProcessor;
use crate::util::server_replay::start_server_replay;

pub struct ServerCommandHandlerReplay {
    game_cache: Arc<Mutex<GameCache>>,
    session_manager: Arc<Mutex<SessionManager>>,
    db_connection_manager: Arc<Mutex<DbConnectionManager>>,
    replayer: Arc<Mutex<ServerReplayer>>,
    /// Java: `getServer().getRequestProcessor()`.
    request_processor: Arc<Mutex<ServerRequestProcessor>>,
    /// Java: `UtilServerHttpClient.fetchPage(...)` inside `ServerRequestLoadReplay.process`.
    client: Arc<dyn HttpClient + Send + Sync>,
    /// Java: `ServerUrlProperty.BACKUP_URL_LOAD.url(server.getProperties())`.
    backup_url_load_template: String,
    /// Java: `server.getCommunication().handleCommand(new ReceivedCommand(...))` — the
    /// redispatch sink `QueuedServerRequestLoadReplay` uses once the backup service answers
    /// with `InternalServerCommandReplayLoaded`, matching the convention established by
    /// `ServerCommandHandlerUploadGame` (Phase AAC).
    dispatch_tx: mpsc::UnboundedSender<ReceivedCommand>,
}

impl ServerCommandHandlerReplay {
    pub fn new(
        game_cache: Arc<Mutex<GameCache>>,
        session_manager: Arc<Mutex<SessionManager>>,
        db_connection_manager: Arc<Mutex<DbConnectionManager>>,
        replayer: Arc<Mutex<ServerReplayer>>,
        request_processor: Arc<Mutex<ServerRequestProcessor>>,
        client: Arc<dyn HttpClient + Send + Sync>,
        backup_url_load_template: impl Into<String>,
        dispatch_tx: mpsc::UnboundedSender<ReceivedCommand>,
    ) -> Self {
        Self {
            game_cache,
            session_manager,
            db_connection_manager,
            replayer,
            request_processor,
            client,
            backup_url_load_template: backup_url_load_template.into(),
            dispatch_tx,
        }
    }

    /// Java: getId() — returns NetCommandId for REPLAY.
    pub fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientReplay
    }

    /// Java: `handleCommand(ReceivedCommand)` — handles a replay request.
    ///
    /// ```java
    /// GameState gameState = getServer().getGameCache().getGameStateById(gameId);
    /// if (gameState == null) {
    ///     gameState = getServer().getGameCache().queryFromDb(gameId);
    ///     getServer().getGameCache().addGame(gameState);
    /// }
    /// if (gameState != null) {
    ///     UtilServerReplay.startServerReplay(gameState, replayToCommandNr, pReceivedCommand.getSession());
    /// } else {
    ///     getServer().getRequestProcessor().add(new ServerRequestLoadReplay(...));
    /// }
    /// ```
    ///
    /// The found-in-cache branch is fully real (Phase AAB): it starts the replay via
    /// `start_server_replay`. The not-found branch's `GameCache::query_from_db` call is also
    /// real (Phase AAA/AAB), but per that method's own doc comment it can only return the raw
    /// gzipped blob bytes (no gunzip-to-`GameState` pipeline exists in this crate), so there is
    /// no reconstructed `GameState` to `addGame` even when the query succeeds — the game
    /// remains "not found" from this handler's point of view. The final fallback enqueues a
    /// `ServerRequestLoadReplay` (mode `LOAD_GAME`) on the `ServerRequestProcessor` via the
    /// `QueuedServerRequestLoadReplay` adapter Phase AAC built — once the backup service
    /// answers, that adapter re-adds the rehydrated `GameState` to the cache and redispatches
    /// `InternalServerCommandReplayLoaded`, which `ServerCommandHandlerReplayLoaded` (Phase AAB)
    /// picks up to actually start the replay.
    pub async fn handle_command(&self, cmd: &ClientCommandReplay, session_id: SessionId) -> bool {
        let mut game_id = cmd.get_game_id();

        if game_id == 0 {
            let sm = self.session_manager.lock().unwrap();
            game_id = sm.get_game_id_for_session(session_id);
        }

        if game_id == 0 {
            return false;
        }

        let message = {
            let gc = self.game_cache.lock().unwrap();
            gc.get_game_state_by_id(game_id)
                .map(|gs| ServerCommandGameState::new(gs.get_game().cloned()).to_json_value().to_string())
        };

        match message {
            Some(message) => {
                let gc = self.game_cache.lock().unwrap();
                let game_state = gc.get_game_state_by_id(game_id).expect("checked above");
                let sm = self.session_manager.lock().unwrap();
                start_server_replay(
                    Some((game_id, message.as_str(), &game_state.game_log)),
                    cmd.get_replay_to_command_nr(),
                    Some(session_id),
                    &sm,
                    &self.replayer,
                );
            }
            None => {
                // Java: `GameCache.queryFromDb(gameId)` — real call, documented gap on
                // reconstructing a `GameState` from its result (see doc comment above).
                let db = self.db_connection_manager.lock().unwrap().clone();
                let _blob = GameCache::query_from_db(&db, game_id).await;
                // Java: `getServer().getRequestProcessor().add(new ServerRequestLoadReplay(...))`.
                let request = ServerRequestLoadReplay::new(
                    game_id,
                    cmd.get_replay_to_command_nr(),
                    ServerRequestLoadReplay::LOAD_GAME,
                    String::new(),
                    cmd.get_coach().unwrap_or("").to_string(),
                );
                let queued = QueuedServerRequestLoadReplay::new(
                    request,
                    Arc::clone(&self.client),
                    self.backup_url_load_template.clone(),
                    Arc::clone(&self.game_cache),
                    self.dispatch_tx.clone(),
                    session_id,
                );
                self.request_processor.lock().unwrap().add(Box::new(queued));
            }
        }

        true
    }
}

impl Default for ServerCommandHandlerReplay {
    fn default() -> Self {
        let (dispatch_tx, _dispatch_rx) = mpsc::unbounded_channel();
        Self::new(
            Arc::new(Mutex::new(GameCache::new())),
            Arc::new(Mutex::new(SessionManager::new())),
            Arc::new(Mutex::new(DbConnectionManager::new())),
            Arc::new(Mutex::new(ServerReplayer::new())),
            Arc::new(Mutex::new(ServerRequestProcessor::new())),
            Arc::new(crate::request::fumbbl::util_fumbbl_request::ReqwestHttpClient::new()),
            String::new(),
            dispatch_tx,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::request::fumbbl::util_fumbbl_request::MockHttpClient;

    fn handler_with(
        game_cache: Arc<Mutex<GameCache>>,
        session_manager: Arc<Mutex<SessionManager>>,
        replayer: Arc<Mutex<ServerReplayer>>,
    ) -> ServerCommandHandlerReplay {
        let (dispatch_tx, _dispatch_rx) = mpsc::unbounded_channel();
        ServerCommandHandlerReplay::new(
            game_cache,
            session_manager,
            Arc::new(Mutex::new(DbConnectionManager::new())),
            replayer,
            Arc::new(Mutex::new(ServerRequestProcessor::new())),
            Arc::new(MockHttpClient { response: Ok(String::new()) }),
            "http://backup/load/$1",
            dispatch_tx,
        )
    }

    #[test]
    fn construct() {
        let _ = ServerCommandHandlerReplay::default();
    }

    #[test]
    fn get_id_returns_client_replay() {
        let h = ServerCommandHandlerReplay::default();
        assert_eq!(h.get_id(), NetCommandId::ClientReplay);
    }

    #[tokio::test]
    async fn handle_command_returns_false_when_no_game_id_available() {
        // Java's default HTTP client (`ReqwestHttpClient`) builds its own tokio runtime, which
        // cannot be constructed-and-dropped from inside a `#[tokio::test]` async context — use
        // the mock-backed helper here, matching the other async tests in this module.
        let h = handler_with(
            Arc::new(Mutex::new(GameCache::new())),
            Arc::new(Mutex::new(SessionManager::new())),
            Arc::new(Mutex::new(ServerReplayer::new())),
        );
        // Command has gameId == 0 and the session (99) was never registered,
        // so getGameIdForSession also returns 0 — Java returns false here.
        let cmd = ClientCommandReplay::new();
        assert!(!h.handle_command(&cmd, 99).await);
    }

    #[tokio::test]
    async fn handle_command_resolves_game_id_from_session_when_command_has_none() {
        use ffb_model::model::ClientMode;

        let gc = Arc::new(Mutex::new(GameCache::new()));
        let sm_arc = Arc::new(Mutex::new(SessionManager::new()));
        {
            let (tx, _rx) = mpsc::unbounded_channel();
            let mut sm = sm_arc.lock().unwrap();
            sm.add_session(1, 42, "Coach".into(), ClientMode::PLAYER, true, vec![], tx);
        }
        let h = handler_with(gc, sm_arc, Arc::new(Mutex::new(ServerReplayer::new())));
        let cmd = ClientCommandReplay::new();
        // Game 42 is not in the cache, and the DB pool is unconfigured (query_from_db
        // returns None per its own doc comment), so the not-found branch now really
        // enqueues a LOAD_GAME ServerRequestLoadReplay on the request processor.
        assert!(h.handle_command(&cmd, 1).await);
        assert_eq!(h.request_processor.lock().unwrap().queue_len(), 1);
    }

    #[tokio::test]
    async fn handle_command_found_game_starts_a_real_replay() {
        let gc = Arc::new(Mutex::new(GameCache::new()));
        let game_id = { gc.lock().unwrap().create_game_state() };
        let sm = Arc::new(Mutex::new(SessionManager::new()));
        let replayer = Arc::new(Mutex::new(ServerReplayer::new()));
        let h = handler_with(Arc::clone(&gc), Arc::clone(&sm), Arc::clone(&replayer));
        let cmd = ClientCommandReplay::with_params(game_id, 0, "coach");
        assert!(h.handle_command(&cmd, 1).await);
        assert_eq!(replayer.lock().unwrap().queue_size(), 1);
    }

    #[tokio::test]
    async fn handle_command_found_game_sends_game_state_to_untracking_session() {
        use ffb_model::model::ClientMode;
        use tokio::sync::mpsc;

        let gc = Arc::new(Mutex::new(GameCache::new()));
        let game_id = { gc.lock().unwrap().create_game_state() };
        let sm = Arc::new(Mutex::new(SessionManager::new()));
        let (tx, mut rx) = mpsc::unbounded_channel();
        sm.lock().unwrap().add_session(1, 0, "Coach".into(), ClientMode::PLAYER, true, vec![], tx);
        let replayer = Arc::new(Mutex::new(ServerReplayer::new()));
        let h = handler_with(Arc::clone(&gc), Arc::clone(&sm), replayer);
        let cmd = ClientCommandReplay::with_params(game_id, 0, "coach");
        assert!(h.handle_command(&cmd, 1).await);
        let msg = rx.try_recv().expect("expected a serverGameState message");
        assert!(msg.contains("serverGameState"));
    }
}
