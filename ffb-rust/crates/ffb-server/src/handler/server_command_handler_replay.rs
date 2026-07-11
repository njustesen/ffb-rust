/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerReplay.
use std::sync::{Arc, Mutex};
use ffb_engine::server_replayer::ServerReplayer;
use ffb_model::enums::NetCommandId;
use ffb_protocol::commands::client_command_replay::ClientCommandReplay;
use ffb_protocol::commands::server_command_game_state::ServerCommandGameState;
use crate::db::db_connection_manager::DbConnectionManager;
use crate::game_cache::GameCache;
use crate::model::received_command::SessionId;
use crate::net::session_manager::SessionManager;
use crate::util::server_replay::start_server_replay;

pub struct ServerCommandHandlerReplay {
    game_cache: Arc<Mutex<GameCache>>,
    session_manager: Arc<Mutex<SessionManager>>,
    db_connection_manager: Arc<Mutex<DbConnectionManager>>,
    replayer: Arc<Mutex<ServerReplayer>>,
}

impl ServerCommandHandlerReplay {
    pub fn new(
        game_cache: Arc<Mutex<GameCache>>,
        session_manager: Arc<Mutex<SessionManager>>,
        db_connection_manager: Arc<Mutex<DbConnectionManager>>,
        replayer: Arc<Mutex<ServerReplayer>>,
    ) -> Self {
        Self { game_cache, session_manager, db_connection_manager, replayer }
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
    /// remains "not found" from this handler's point of view. The final fallback (enqueueing
    /// `ServerRequestLoadReplay` on the `ServerRequestProcessor`) stays a documented `todo!()`:
    /// `ServerRequestLoadReplay` does not implement the `ServerRequest` trait yet (its own
    /// `process` takes an `HttpClient` + URL template, not `ServerRequest::process(&self)`),
    /// so it cannot be queued — that trait-signature fix is Phase AAC's scope, not this one's.
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
                todo!(
                    "Phase AAC: ServerRequestLoadReplay does not implement the ServerRequest \
                     trait yet, so it cannot be queued on ServerRequestProcessor here"
                )
            }
        }

        true
    }
}

impl Default for ServerCommandHandlerReplay {
    fn default() -> Self {
        Self::new(
            Arc::new(Mutex::new(GameCache::new())),
            Arc::new(Mutex::new(SessionManager::new())),
            Arc::new(Mutex::new(DbConnectionManager::new())),
            Arc::new(Mutex::new(ServerReplayer::new())),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let h = ServerCommandHandlerReplay::default();
        // Command has gameId == 0 and the session (99) was never registered,
        // so getGameIdForSession also returns 0 — Java returns false here.
        let cmd = ClientCommandReplay::new();
        assert!(!h.handle_command(&cmd, 99).await);
    }

    #[tokio::test]
    #[should_panic(expected = "ServerRequestLoadReplay")]
    async fn handle_command_resolves_game_id_from_session_when_command_has_none() {
        use ffb_model::model::ClientMode;
        use tokio::sync::mpsc;

        let gc = Arc::new(Mutex::new(GameCache::new()));
        let sm_arc = Arc::new(Mutex::new(SessionManager::new()));
        {
            let (tx, _rx) = mpsc::unbounded_channel();
            let mut sm = sm_arc.lock().unwrap();
            sm.add_session(1, 42, "Coach".into(), ClientMode::PLAYER, true, vec![], tx);
        }
        let h = ServerCommandHandlerReplay::new(
            gc,
            sm_arc,
            Arc::new(Mutex::new(DbConnectionManager::new())),
            Arc::new(Mutex::new(ServerReplayer::new())),
        );
        let cmd = ClientCommandReplay::new();
        // Game 42 is not in the cache, and the DB pool is unconfigured (query_from_db
        // returns None per its own doc comment), so the not-found branch's queue-enqueue
        // todo!() is reached — this is a narrow, documented gap blocked on Phase AAC.
        h.handle_command(&cmd, 1).await;
    }

    #[tokio::test]
    async fn handle_command_found_game_starts_a_real_replay() {
        let gc = Arc::new(Mutex::new(GameCache::new()));
        let game_id = { gc.lock().unwrap().create_game_state() };
        let sm = Arc::new(Mutex::new(SessionManager::new()));
        let replayer = Arc::new(Mutex::new(ServerReplayer::new()));
        let h = ServerCommandHandlerReplay::new(
            Arc::clone(&gc),
            Arc::clone(&sm),
            Arc::new(Mutex::new(DbConnectionManager::new())),
            Arc::clone(&replayer),
        );
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
        let h = ServerCommandHandlerReplay::new(
            Arc::clone(&gc),
            Arc::clone(&sm),
            Arc::new(Mutex::new(DbConnectionManager::new())),
            replayer,
        );
        let cmd = ClientCommandReplay::with_params(game_id, 0, "coach");
        assert!(h.handle_command(&cmd, 1).await);
        let msg = rx.try_recv().expect("expected a serverGameState message");
        assert!(msg.contains("serverGameState"));
    }
}
