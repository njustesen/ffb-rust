/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerCloseGame.
use std::sync::{Arc, Mutex};
use ffb_model::enums::NetCommandId;
use crate::game_cache::GameCache;
use crate::net::commands::internal_server_command::InternalServerCommand;
use crate::net::commands::internal_server_command_close_game::InternalServerCommandCloseGame;
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
    /// `GameCache.closeGame(long)` (Java): if `gameId <= 0` or the game isn't
    /// cached, does nothing. Otherwise it closes every session for the game
    /// (mirrored here via `SessionManager.remove_session`) and then removes
    /// the `GameState` from the cache and — in FUMBBL mode — dispatches a
    /// `FumbblRequestRemoveGamestate`. `GameCache` has no removal API in the
    /// Rust in-memory MVP yet, and the FUMBBL request pipeline isn't wired.
    pub fn handle_command(&self, close_game_command: &InternalServerCommandCloseGame) -> bool {
        let game_id = close_game_command.get_game_id();
        if game_id <= 0 {
            return true;
        }
        let found = {
            let gc = self.game_cache.lock().unwrap();
            gc.get_game_state_by_id(game_id).is_some()
        };
        if !found {
            return true;
        }
        {
            let mut sm = self.session_manager.lock().unwrap();
            for session_id in sm.get_sessions_for_game_id(game_id) {
                sm.remove_session(session_id);
            }
        }
        // Java: `removeGame(gameId)` + (FUMBBL mode) `FumbblRequestRemoveGamestate`.
        todo!(
            "Phase ZV: GameCache::remove_game + FumbblRequestRemoveGamestate not wired (game_id = {})",
            game_id
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::ClientMode;
    use tokio::sync::mpsc;

    fn setup() -> (Arc<Mutex<GameCache>>, Arc<Mutex<SessionManager>>) {
        (
            Arc::new(Mutex::new(GameCache::new())),
            Arc::new(Mutex::new(SessionManager::new())),
        )
    }

    #[test]
    fn construct() {
        let (gc, sm) = setup();
        let _ = ServerCommandHandlerCloseGame::new(gc, sm);
    }

    #[test]
    fn get_id_is_internal_server_close_game() {
        let (gc, sm) = setup();
        let handler = ServerCommandHandlerCloseGame::new(gc, sm);
        assert_eq!(handler.get_id(), NetCommandId::InternalServerCloseGame);
    }

    #[test]
    fn zero_game_id_returns_true_without_touching_sessions() {
        let (gc, sm) = setup();
        let handler = ServerCommandHandlerCloseGame::new(gc, sm);
        let cmd = InternalServerCommandCloseGame::new(0);
        assert!(handler.handle_command(&cmd));
    }

    #[test]
    fn unknown_game_id_returns_true() {
        let (gc, sm) = setup();
        let handler = ServerCommandHandlerCloseGame::new(gc, sm);
        let cmd = InternalServerCommandCloseGame::new(999);
        assert!(handler.handle_command(&cmd));
    }

    #[test]
    fn known_game_id_removes_sessions_before_hitting_missing_cache_removal() {
        let (gc, sm) = setup();
        let game_id = gc.lock().unwrap().create_game_state();
        {
            let (tx, _rx) = mpsc::unbounded_channel();
            sm.lock().unwrap().add_session(1, game_id, "Coach".into(), ClientMode::PLAYER, true, vec![], tx);
        }
        let handler = ServerCommandHandlerCloseGame::new(Arc::clone(&gc), Arc::clone(&sm));
        let cmd = InternalServerCommandCloseGame::new(game_id);
        let result = std::panic::catch_unwind(|| handler.handle_command(&cmd));
        assert!(result.is_err());
        // The session should have been closed before the (currently-missing)
        // cache-removal step panics.
        assert!(sm.lock().unwrap().get_sessions_for_game_id(game_id).is_empty());
    }
}
