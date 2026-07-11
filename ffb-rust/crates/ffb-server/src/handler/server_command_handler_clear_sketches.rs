/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerClearSketches.
///
/// Java: `extends AbstractServerCommandHandlerSketch<ClientCommandClearSketches, ServerCommandClearSketches>`.
/// `AbstractServerCommandHandlerSketch` (this crate) is a generic composition helper
/// (`SketchCommandOps`), but its `handle_command` dispatch is re-implemented directly here
/// against this handler's own `sketch_manager` / `replay_session_manager`, matching the Java
/// source line for line — this mirrors how the sibling sketch handlers are structured.
use std::sync::{Arc, Mutex};

use ffb_engine::server_sketch_manager::ServerSketchManager;
use ffb_model::enums::NetCommandId;
use ffb_protocol::commands::client_command_clear_sketches::ClientCommandClearSketches;
use ffb_protocol::commands::server_command_clear_sketches::ServerCommandClearSketches;

use crate::model::received_command::SessionId;
use crate::net::replay_session_manager::ReplaySessionManager;

pub struct ServerCommandHandlerClearSketches {
    /// Java: `sketchManager` (inherited from `AbstractServerCommandHandlerSketch`).
    sketch_manager: Arc<Mutex<ServerSketchManager>>,
    /// Java: `replaySessionManager` (inherited from `AbstractServerCommandHandlerSketch`).
    replay_session_manager: Arc<Mutex<ReplaySessionManager>>,
}

impl ServerCommandHandlerClearSketches {
    pub fn new(
        sketch_manager: Arc<Mutex<ServerSketchManager>>,
        replay_session_manager: Arc<Mutex<ReplaySessionManager>>,
    ) -> Self {
        Self { sketch_manager, replay_session_manager }
    }

    /// Java: `getId()` — returns `NetCommandId.CLIENT_CLEAR_SKETCHES`.
    pub fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientClearSketches
    }

    /// Java: `updateSketchManager(Session, ClientCommandClearSketches)` —
    /// `sketchManager.remove(session); replaySessionManager.otherSessions(session).forEach(sketchManager::remove);`
    pub fn update_sketch_manager(
        &self,
        session_id: SessionId,
        _command: &ClientCommandClearSketches,
    ) {
        let others = { self.replay_session_manager.lock().unwrap().other_sessions(session_id) };
        let mut mgr = self.sketch_manager.lock().unwrap();
        mgr.remove_session(&session_id.to_string());
        for other in others {
            mgr.remove_session(&other.to_string());
        }
    }

    /// Java: `createServerCommand(String, ClientCommandClearSketches)` —
    /// `new ServerCommandClearSketches()`.
    pub fn create_server_command(
        &self,
        _coach: &str,
        _command: &ClientCommandClearSketches,
    ) -> ServerCommandClearSketches {
        ServerCommandClearSketches::new()
    }

    /// Java: `getServer().getCommunication().sendToReplaySession(otherSession, serverCommand)`'s
    /// wire payload. `ServerCommandClearSketches` carries no fields.
    fn to_json() -> String {
        format!("{{\"netCommandId\":\"{}\"}}", NetCommandId::ServerClearSketches.name())
    }

    /// Java: `AbstractServerCommandHandlerSketch.handleCommand(ReceivedCommand)` (final, inherited).
    pub fn handle_command(
        &self,
        session_id: SessionId,
        command: &ClientCommandClearSketches,
    ) -> bool {
        let has = { self.replay_session_manager.lock().unwrap().has(session_id) };
        if has {
            // Java: `command.requiresControl()` is always `false` for `ClientSketchCommand`
            // subclasses, so this guard never actually returns early — translated for fidelity.
            let requires_control = false;
            let has_control = { self.replay_session_manager.lock().unwrap().has_control(session_id) };
            if requires_control && !has_control {
                return true;
            }
            self.update_sketch_manager(session_id, command);
            let coach = { self.replay_session_manager.lock().unwrap().coach(session_id) }
                .unwrap_or_default();
            let others = { self.replay_session_manager.lock().unwrap().other_sessions(session_id) };
            for other_session in others {
                let _server_command = self.create_server_command(&coach, command);
                // Java: `getServer().getCommunication().sendToReplaySession(otherSession, serverCommand)`.
                let json = Self::to_json();
                self.replay_session_manager.lock().unwrap().send_to(other_session, &json);
            }
        }
        true
    }
}

impl Default for ServerCommandHandlerClearSketches {
    fn default() -> Self {
        Self::new(
            Arc::new(Mutex::new(ServerSketchManager::new())),
            Arc::new(Mutex::new(ReplaySessionManager::new())),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_engine::server_sketch_manager::Sketch as ManagerSketch;

    #[test]
    fn construct() {
        let _ = ServerCommandHandlerClearSketches::default();
    }

    #[test]
    fn get_id_returns_client_clear_sketches() {
        let handler = ServerCommandHandlerClearSketches::default();
        assert_eq!(handler.get_id(), NetCommandId::ClientClearSketches);
    }

    #[test]
    fn update_sketch_manager_removes_own_and_other_sessions() {
        let handler = ServerCommandHandlerClearSketches::default();
        {
            let mut mgr = handler.sketch_manager.lock().unwrap();
            mgr.add_sketch("1", ManagerSketch::new("sk-1"));
            mgr.add_sketch("2", ManagerSketch::new("sk-2"));
        }
        {
            let mut rsm = handler.replay_session_manager.lock().unwrap();
            rsm.add_session(1, "replay".to_string(), "Alice".to_string());
            rsm.add_session(2, "replay".to_string(), "Bob".to_string());
        }
        handler.update_sketch_manager(1, &ClientCommandClearSketches::new());
        let mut mgr = handler.sketch_manager.lock().unwrap();
        assert!(mgr.get_sketches("1").is_empty());
        assert!(mgr.get_sketches("2").is_empty());
    }

    #[test]
    fn create_server_command_ignores_inputs() {
        let handler = ServerCommandHandlerClearSketches::default();
        let cmd = handler.create_server_command("Alice", &ClientCommandClearSketches::new());
        assert_eq!(format!("{cmd:?}"), format!("{:?}", ServerCommandClearSketches::new()));
    }

    #[test]
    fn handle_command_without_replay_session_is_noop() {
        let handler = ServerCommandHandlerClearSketches::default();
        {
            let mut mgr = handler.sketch_manager.lock().unwrap();
            mgr.add_sketch("1", ManagerSketch::new("sk-1"));
        }
        assert!(handler.handle_command(1, &ClientCommandClearSketches::new()));
        let mut mgr = handler.sketch_manager.lock().unwrap();
        assert_eq!(mgr.get_sketches("1").len(), 1);
    }

    #[test]
    fn handle_command_with_replay_session_and_no_others_clears_own_sketches() {
        let handler = ServerCommandHandlerClearSketches::default();
        {
            let mut mgr = handler.sketch_manager.lock().unwrap();
            mgr.add_sketch("1", ManagerSketch::new("sk-1"));
        }
        {
            let mut rsm = handler.replay_session_manager.lock().unwrap();
            rsm.add_session(1, "replay".to_string(), "Alice".to_string());
        }
        assert!(handler.handle_command(1, &ClientCommandClearSketches::new()));
        let mut mgr = handler.sketch_manager.lock().unwrap();
        assert!(mgr.get_sketches("1").is_empty());
    }

    #[test]
    fn handle_command_with_other_replay_sessions_delivers_to_registered_sender() {
        use tokio::sync::mpsc;
        let handler = ServerCommandHandlerClearSketches::default();
        let (tx, mut rx) = mpsc::unbounded_channel();
        {
            let mut rsm = handler.replay_session_manager.lock().unwrap();
            rsm.add_session(1, "replay".to_string(), "Alice".to_string());
            rsm.add_session(2, "replay".to_string(), "Bob".to_string());
            rsm.register_sender(2, tx);
        }
        assert!(handler.handle_command(1, &ClientCommandClearSketches::new()));
        let sent = rx.try_recv().expect("expected a message forwarded to session 2");
        assert!(sent.contains("serverClearSketches"));
    }
}
