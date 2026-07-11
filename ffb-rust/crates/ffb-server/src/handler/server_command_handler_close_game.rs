/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerCloseGame.
use std::sync::{Arc, Mutex};
use ffb_model::enums::NetCommandId;
use crate::db::db_connection_manager::DbConnectionManager;
use crate::game_cache::GameCache;
use crate::net::commands::internal_server_command::InternalServerCommand;
use crate::net::commands::internal_server_command_close_game::InternalServerCommandCloseGame;
use crate::net::server_communication::ServerCommunication;
use crate::net::session_manager::SessionManager;

/// Java: `ServerCommandHandlerCloseGame extends ServerCommandHandler`.
pub struct ServerCommandHandlerCloseGame {
    game_cache: Arc<Mutex<GameCache>>,
    session_manager: Arc<Mutex<SessionManager>>,
}

impl ServerCommandHandlerCloseGame {
    pub fn new(game_cache: Arc<Mutex<GameCache>>, session_manager: Arc<Mutex<SessionManager>>) -> Self {
        Self { game_cache, session_manager }
    }

    /// Java: `getId()` — returns `NetCommandId.INTERNAL_SERVER_CLOSE_GAME`.
    pub fn get_id(&self) -> NetCommandId {
        NetCommandId::InternalServerCloseGame
    }

    /// Java: `handleCommand(ReceivedCommand)`.
    ///
    /// ```java
    /// InternalServerCommandCloseGame closeGameCommand = (InternalServerCommandCloseGame) pReceivedCommand.getCommand();
    /// getServer().getGameCache().closeGame(closeGameCommand.getGameId());
    /// return true;
    /// ```
    ///
    /// `GameCache::close_game` (see `game_cache.rs`, added in Phase ZX.2) now performs the
    /// real, portable half of Java's `GameCache.closeGame(long)`: the `gameId <= 0` / not-cached
    /// short-circuits, closing every session tracking the game, and removing it from the
    /// cache. Java's `removeGame` private helper also decides whether to `queueDbDelete`
    /// (only one team joined / never started / test game); `GameCache::close_game` reports
    /// that decision back as `Some(should_queue_db_delete)`, which is now acted on for real
    /// via `GameCache::queue_db_delete`.
    ///
    /// What remains unported: Java's mode-dependent tail —
    /// `(FUMBBL mode && status not REPLAYING/LOADING) -> enqueue FumbblRequestRemoveGamestate`
    /// else `fServer.closeResources(gameId)` — has no Rust equivalent (no `FumbblRequestRemoveGamestate`,
    /// no `ServerMode`-gated `closeResources`), so it's a documented no-op rather than a `todo!()`.
    pub async fn handle_command(
        &self,
        close_game_command: &InternalServerCommandCloseGame,
        communication: &ServerCommunication,
        db: &DbConnectionManager,
    ) -> bool {
        let game_id = close_game_command.get_game_id();

        // `communication.close(session_id)` re-locks `SessionManager` internally (see
        // `net/server_communication.rs`); in real wiring `communication`'s session manager
        // and `self.session_manager` are the *same* shared `Arc<Mutex<_>>`, so every
        // session is closed here first, with no `SessionManager` guard of ours held across
        // the call. By the time `GameCache::close_game` runs below, its own internal
        // `communication.close(...)` loop finds zero sessions left for this game and never
        // re-enters `communication`'s lock — avoiding a self-deadlock on the shared mutex.
        let session_ids = {
            let sm = self.session_manager.lock().unwrap();
            sm.get_sessions_for_game_id(game_id)
        };
        for session_id in session_ids {
            communication.close(session_id);
        }

        let should_queue_db_delete = {
            let mut gc = self.game_cache.lock().unwrap();
            let sm = self.session_manager.lock().unwrap();
            gc.close_game(game_id, &sm, communication)
        };

        if let Some(true) = should_queue_db_delete {
            let _ = GameCache::queue_db_delete(db, game_id, true).await;
        }

        // Java: `(FUMBBL mode && status != REPLAYING/LOADING) -> FumbblRequestRemoveGamestate`
        // else `fServer.closeResources(gameId)` — no Rust equivalent, documented gap.
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::ClientMode;
    use tokio::sync::mpsc;

    fn setup() -> (Arc<Mutex<GameCache>>, Arc<Mutex<SessionManager>>, ServerCommunication, DbConnectionManager) {
        let gc = Arc::new(Mutex::new(GameCache::new()));
        let sm = Arc::new(Mutex::new(SessionManager::new()));
        let db = Arc::new(Mutex::new(DbConnectionManager::new()));
        let communication = ServerCommunication::new(Arc::clone(&gc), Arc::clone(&sm), Arc::clone(&db));
        (gc, sm, communication, DbConnectionManager::new())
    }

    #[tokio::test]
    async fn construct() {
        let (gc, sm, _comm, _db) = setup();
        let _ = ServerCommandHandlerCloseGame::new(gc, sm);
    }

    #[tokio::test]
    async fn get_id_is_internal_server_close_game() {
        let (gc, sm, _comm, _db) = setup();
        let handler = ServerCommandHandlerCloseGame::new(gc, sm);
        assert_eq!(handler.get_id(), NetCommandId::InternalServerCloseGame);
    }

    #[tokio::test]
    async fn zero_game_id_returns_true_without_touching_sessions() {
        let (gc, sm, comm, db) = setup();
        let handler = ServerCommandHandlerCloseGame::new(gc, sm);
        let cmd = InternalServerCommandCloseGame::new(0);
        assert!(handler.handle_command(&cmd, &comm, &db).await);
    }

    #[tokio::test]
    async fn unknown_game_id_returns_true() {
        let (gc, sm, comm, db) = setup();
        let handler = ServerCommandHandlerCloseGame::new(gc, sm);
        let cmd = InternalServerCommandCloseGame::new(999);
        assert!(handler.handle_command(&cmd, &comm, &db).await);
    }

    #[tokio::test]
    async fn known_game_id_removes_sessions_and_the_game_from_the_cache() {
        let (gc, sm, comm, db) = setup();
        let game_id = gc.lock().unwrap().create_game_state();
        {
            let (tx, _rx) = mpsc::unbounded_channel();
            sm.lock().unwrap().add_session(1, game_id, "Coach".into(), ClientMode::PLAYER, true, vec![], tx);
        }
        let handler = ServerCommandHandlerCloseGame::new(Arc::clone(&gc), Arc::clone(&sm));
        let cmd = InternalServerCommandCloseGame::new(game_id);
        assert!(handler.handle_command(&cmd, &comm, &db).await);

        assert!(sm.lock().unwrap().get_sessions_for_game_id(game_id).is_empty());
        assert!(gc.lock().unwrap().get_game_state_by_id(game_id).is_none());
    }

    #[tokio::test]
    async fn queue_db_delete_without_pool_configured_is_a_noop() {
        // A freshly-created GameState (no started Game) is treated as never-started, so
        // `GameCache::close_game` reports `Some(true)` (should queue a DB delete); with no
        // DB pool configured, `queue_db_delete` degrades to a no-op instead of panicking.
        let (gc, sm, comm, db) = setup();
        let game_id = gc.lock().unwrap().create_game_state();
        let handler = ServerCommandHandlerCloseGame::new(Arc::clone(&gc), Arc::clone(&sm));
        let cmd = InternalServerCommandCloseGame::new(game_id);
        assert!(handler.handle_command(&cmd, &comm, &db).await);
    }
}
