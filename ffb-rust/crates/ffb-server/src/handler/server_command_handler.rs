/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandler.
///
/// Java: abstract base class holding a reference to `FantasyFootballServer` and exposing
/// `getServer()` / `isServerInTestMode()` to concrete subclasses; `getId()` is `abstract`
/// (no method body) and is therefore implemented individually by each concrete
/// `ServerCommandHandlerXxx` struct rather than here.
///
/// Rust has no class inheritance, so this struct plays the role of Java's `fServer` field via
/// composition: it holds the two subsystems concrete handlers actually need — the shared
/// `GameCache` and `SessionManager` — mirroring the same composition already used by
/// `ServerCommandHandlerFactory`. Concrete handler structs embed a `ServerCommandHandler` (or
/// hold the same `Arc<Mutex<_>>` handles directly) instead of extending it.
use std::sync::{Arc, Mutex};

use crate::game_cache::GameCache;
use crate::net::session_manager::SessionManager;

/// Java: `ServerCommandHandler`.
pub struct ServerCommandHandler {
    /// Java: `fServer` — decomposed into the two subsystems actually used by subclasses.
    game_cache: Arc<Mutex<GameCache>>,
    session_manager: Arc<Mutex<SessionManager>>,
    /// Java: `getServer().getProperty(IServerProperty.SERVER_TEST)` combined with
    /// `Boolean.parseBoolean(...)`, resolved once at construction time since no dynamic
    /// server-property store is wired into `ffb-server` yet (Phase ZV).
    server_test_mode: bool,
}

impl ServerCommandHandler {
    /// Java: `protected ServerCommandHandler(FantasyFootballServer pServer)`.
    pub fn new(
        game_cache: Arc<Mutex<GameCache>>,
        session_manager: Arc<Mutex<SessionManager>>,
        server_test_mode: bool,
    ) -> Self {
        Self {
            game_cache,
            session_manager,
            server_test_mode,
        }
    }

    /// Java: `getServer()` — returns the `FantasyFootballServer` instance.
    ///
    /// The Rust translation exposes the shared `GameCache` handle instead of a monolithic
    /// server reference; see the type-level doc comment.
    pub fn game_cache(&self) -> &Arc<Mutex<GameCache>> {
        &self.game_cache
    }

    /// Java: `getServer()` — the `SessionManager` half of the same accessor.
    pub fn session_manager(&self) -> &Arc<Mutex<SessionManager>> {
        &self.session_manager
    }

    /// Java: `isServerInTestMode()`.
    pub fn is_server_in_test_mode(&self) -> bool {
        self.server_test_mode
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let gc = Arc::new(Mutex::new(GameCache::new()));
        let sm = Arc::new(Mutex::new(SessionManager::new()));
        let _ = ServerCommandHandler::new(gc, sm, false);
    }

    #[test]
    fn exposes_shared_game_cache() {
        let gc = Arc::new(Mutex::new(GameCache::new()));
        let sm = Arc::new(Mutex::new(SessionManager::new()));
        let handler = ServerCommandHandler::new(Arc::clone(&gc), sm, false);
        let id = handler.game_cache().lock().unwrap().create_game_state();
        assert!(gc.lock().unwrap().get_game_state_by_id(id).is_some());
    }

    #[test]
    fn exposes_shared_session_manager() {
        use ffb_model::model::ClientMode;
        use tokio::sync::mpsc;

        let gc = Arc::new(Mutex::new(GameCache::new()));
        let sm = Arc::new(Mutex::new(SessionManager::new()));
        let handler = ServerCommandHandler::new(gc, Arc::clone(&sm), false);
        let (tx, _rx) = mpsc::unbounded_channel();
        handler
            .session_manager()
            .lock()
            .unwrap()
            .add_session(1, 100, "Coach".into(), ClientMode::PLAYER, true, vec![], tx);
        assert_eq!(sm.lock().unwrap().get_coach_for_session(1), Some("Coach"));
    }

    #[test]
    fn is_server_in_test_mode_reflects_constructor_flag() {
        let gc = Arc::new(Mutex::new(GameCache::new()));
        let sm = Arc::new(Mutex::new(SessionManager::new()));
        let handler = ServerCommandHandler::new(gc, sm, true);
        assert!(handler.is_server_in_test_mode());
    }

    #[test]
    fn is_server_in_test_mode_false_by_default_flag() {
        let gc = Arc::new(Mutex::new(GameCache::new()));
        let sm = Arc::new(Mutex::new(SessionManager::new()));
        let handler = ServerCommandHandler::new(gc, sm, false);
        assert!(!handler.is_server_in_test_mode());
    }
}
