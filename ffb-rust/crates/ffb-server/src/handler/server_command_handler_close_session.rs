/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerCloseSession.
use ffb_model::enums::NetCommandId;

use crate::handler::server_command_handler_socket_closed::ServerCommandHandlerSocketClosed;
use crate::model::received_command::SessionId;

/// Java: `ServerCommandHandlerCloseSession`.
pub struct ServerCommandHandlerCloseSession {
    /// Java: `handleCommand` delegates to
    /// `getServer().getCommunication().close(pReceivedCommand.getSession())`, which itself
    /// (1) closes the raw WebSocket session, then (2) re-enters `handleCommand` with a
    /// synthetic `InternalServerCommandSocketClosed`, dispatched to
    /// `ServerCommandHandlerSocketClosed`. The Rust translation models both steps as one
    /// delegated call: `SessionManager::remove_session` (invoked by the socket-closed
    /// handler's cleanup) drops the session's outgoing sender, which is what ends the
    /// WebSocket task's select loop in `net::command_socket` — the same practical effect as
    /// Java's `Session.close()`.
    socket_closed: ServerCommandHandlerSocketClosed,
}

impl ServerCommandHandlerCloseSession {
    /// Java: `protected ServerCommandHandlerCloseSession(FantasyFootballServer pServer)`.
    pub fn new(socket_closed: ServerCommandHandlerSocketClosed) -> Self {
        Self { socket_closed }
    }

    /// Java: `getId()` — returns `NetCommandId.CLIENT_CLOSE_SESSION`.
    pub fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientCloseSession
    }

    /// Java: `handleCommand(ReceivedCommand)`.
    pub fn handle_command(&self, session_id: SessionId) -> bool {
        self.socket_closed.handle_command(session_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_engine::server_sketch_manager::ServerSketchManager;
    use ffb_model::model::ClientMode;
    use std::sync::{Arc, Mutex};
    use tokio::sync::mpsc;

    use crate::game_cache::GameCache;
    use crate::net::replay_session_manager::ReplaySessionManager;
    use crate::net::session_manager::SessionManager;

    fn handler(
        session_manager: Arc<Mutex<SessionManager>>,
        game_cache: Arc<Mutex<GameCache>>,
    ) -> ServerCommandHandlerCloseSession {
        let socket_closed = ServerCommandHandlerSocketClosed::new(
            game_cache,
            session_manager,
            Arc::new(Mutex::new(ReplaySessionManager::new())),
            Arc::new(Mutex::new(ServerSketchManager::new())),
        );
        ServerCommandHandlerCloseSession::new(socket_closed)
    }

    #[test]
    fn get_id_is_client_close_session() {
        let h = handler(
            Arc::new(Mutex::new(SessionManager::new())),
            Arc::new(Mutex::new(GameCache::new())),
        );
        assert_eq!(h.get_id(), NetCommandId::ClientCloseSession);
    }

    #[test]
    fn handle_command_removes_session_bookkeeping() {
        let sm = Arc::new(Mutex::new(SessionManager::new()));
        let gc = Arc::new(Mutex::new(GameCache::new()));
        let game_id = gc.lock().unwrap().create_game_state();
        let (tx, _rx) = mpsc::unbounded_channel();
        sm.lock()
            .unwrap()
            .add_session(1, game_id, "Coach".into(), ClientMode::PLAYER, true, vec![], tx);

        let h = handler(Arc::clone(&sm), gc);
        let ok = h.handle_command(1);

        assert!(ok);
        assert_eq!(sm.lock().unwrap().get_game_id_for_session(1), 0);
    }

    #[test]
    fn handle_command_drops_sender_ending_the_transport() {
        let sm = Arc::new(Mutex::new(SessionManager::new()));
        let gc = Arc::new(Mutex::new(GameCache::new()));
        let game_id = gc.lock().unwrap().create_game_state();
        let (tx, mut rx) = mpsc::unbounded_channel();
        sm.lock()
            .unwrap()
            .add_session(1, game_id, "Coach".into(), ClientMode::PLAYER, true, vec![], tx);

        let h = handler(Arc::clone(&sm), gc);
        h.handle_command(1);

        // The only sender for session 1 was owned by SessionManager; removing the session
        // drops it, so the receiving end (as owned by a WebSocket task) observes channel close.
        assert!(rx.try_recv().is_err());
        assert!(sm.lock().unwrap().get_coach_for_session(1).is_none());
    }
}
