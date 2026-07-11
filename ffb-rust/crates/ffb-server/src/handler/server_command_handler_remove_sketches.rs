/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerRemoveSketches.
///
/// Java: `extends AbstractServerCommandHandlerSketch<ClientCommandRemoveSketches, ServerCommandRemoveSketches>`.
/// `AbstractServerCommandHandlerSketch` (this crate) is a generic composition helper
/// (`SketchCommandOps`), but its `handle_command` dispatch is re-implemented directly here
/// against this handler's own `sketch_manager` / `replay_session_manager`, matching the Java
/// source line for line — this mirrors how the sibling sketch handlers are structured.
use std::sync::{Arc, Mutex};

use ffb_engine::server_sketch_manager::ServerSketchManager;
use ffb_model::enums::NetCommandId;
use ffb_protocol::commands::client_command_remove_sketches::ClientCommandRemoveSketches;
use ffb_protocol::commands::server_command_remove_sketches::ServerCommandRemoveSketches;

use crate::model::received_command::SessionId;
use crate::net::replay_session_manager::ReplaySessionManager;

pub struct ServerCommandHandlerRemoveSketches {
    /// Java: `sketchManager` (inherited from `AbstractServerCommandHandlerSketch`).
    sketch_manager: Arc<Mutex<ServerSketchManager>>,
    /// Java: `replaySessionManager` (inherited from `AbstractServerCommandHandlerSketch`).
    replay_session_manager: Arc<Mutex<ReplaySessionManager>>,
}

impl ServerCommandHandlerRemoveSketches {
    pub fn new(
        sketch_manager: Arc<Mutex<ServerSketchManager>>,
        replay_session_manager: Arc<Mutex<ReplaySessionManager>>,
    ) -> Self {
        Self { sketch_manager, replay_session_manager }
    }

    /// Java: `getId()` — returns `NetCommandId.CLIENT_REMOVE_SKETCHES`.
    pub fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientRemoveSketches
    }

    /// Java: `updateSketchManager(Session, ClientCommandRemoveSketches)` —
    /// `if (command.getIds() == null || command.getIds().isEmpty()) sketchManager.remove(session);`
    /// `else sketchManager.removeSketches(session, command.getIds());`
    pub fn update_sketch_manager(
        &self,
        session_id: SessionId,
        command: &ClientCommandRemoveSketches,
    ) {
        let mut mgr = self.sketch_manager.lock().unwrap();
        if command.get_ids().is_empty() {
            mgr.remove_session(&session_id.to_string());
        } else {
            let ids: Vec<&str> = command.get_ids().iter().map(String::as_str).collect();
            mgr.remove_sketches(&session_id.to_string(), &ids);
        }
    }

    /// Java: `createServerCommand(String, ClientCommandRemoveSketches)` —
    /// `new ServerCommandRemoveSketches(coach, command.getIds())`.
    pub fn create_server_command(
        &self,
        coach: &str,
        command: &ClientCommandRemoveSketches,
    ) -> ServerCommandRemoveSketches {
        ServerCommandRemoveSketches::new(coach, command.get_ids().to_vec())
    }

    /// Java: `getServer().getCommunication().sendToReplaySession(otherSession, serverCommand)`'s
    /// wire payload. `ServerCommandRemoveSketches` has no serde impl of its own (it isn't a
    /// `ffb_protocol::server_commands::ServerCommand` variant), so the JSON is built directly
    /// from its fields, same as `ServerCommandHandlerReplayLoaded`'s `ServerCommandStatus`.
    fn to_json(command: &ServerCommandRemoveSketches) -> String {
        format!(
            "{{\"netCommandId\":\"{}\",\"coach\":{:?},\"ids\":{}}}",
            NetCommandId::ServerRemoveSketches.name(),
            command.get_coach(),
            serde_json::to_string(command.get_ids()).unwrap_or_else(|_| "[]".to_string())
        )
    }

    /// Java: `AbstractServerCommandHandlerSketch.handleCommand(ReceivedCommand)` (final, inherited).
    pub fn handle_command(
        &self,
        session_id: SessionId,
        command: &ClientCommandRemoveSketches,
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
                let server_command = self.create_server_command(&coach, command);
                // Java: `getServer().getCommunication().sendToReplaySession(otherSession, serverCommand)`.
                let json = Self::to_json(&server_command);
                self.replay_session_manager.lock().unwrap().send_to(other_session, &json);
            }
        }
        true
    }
}

impl Default for ServerCommandHandlerRemoveSketches {
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
        let _ = ServerCommandHandlerRemoveSketches::default();
    }

    #[test]
    fn get_id_returns_client_remove_sketches() {
        let handler = ServerCommandHandlerRemoveSketches::default();
        assert_eq!(handler.get_id(), NetCommandId::ClientRemoveSketches);
    }

    #[test]
    fn update_sketch_manager_empty_ids_removes_whole_session() {
        let handler = ServerCommandHandlerRemoveSketches::default();
        {
            let mut mgr = handler.sketch_manager.lock().unwrap();
            mgr.add_sketch("1", ManagerSketch::new("sk-1"));
        }
        handler.update_sketch_manager(1, &ClientCommandRemoveSketches::new());
        let mut mgr = handler.sketch_manager.lock().unwrap();
        assert!(mgr.get_sketches("1").is_empty());
    }

    #[test]
    fn update_sketch_manager_with_ids_removes_matching_only() {
        let handler = ServerCommandHandlerRemoveSketches::default();
        {
            let mut mgr = handler.sketch_manager.lock().unwrap();
            mgr.add_sketch("1", ManagerSketch::new("sk-1"));
            mgr.add_sketch("1", ManagerSketch::new("sk-2"));
        }
        handler.update_sketch_manager(
            1,
            &ClientCommandRemoveSketches::with_ids(vec!["sk-1".to_string()]),
        );
        let mut mgr = handler.sketch_manager.lock().unwrap();
        let remaining = mgr.get_sketches("1");
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].get_id(), "sk-2");
    }

    #[test]
    fn create_server_command_carries_coach_and_ids() {
        let handler = ServerCommandHandlerRemoveSketches::default();
        let command = ClientCommandRemoveSketches::with_ids(vec!["sk-1".to_string()]);
        let server_command = handler.create_server_command("Alice", &command);
        assert_eq!(server_command.get_coach(), "Alice");
        assert_eq!(server_command.get_ids(), &["sk-1".to_string()]);
    }

    #[test]
    fn handle_command_without_replay_session_is_noop() {
        let handler = ServerCommandHandlerRemoveSketches::default();
        {
            let mut mgr = handler.sketch_manager.lock().unwrap();
            mgr.add_sketch("1", ManagerSketch::new("sk-1"));
        }
        assert!(handler.handle_command(1, &ClientCommandRemoveSketches::new()));
        let mut mgr = handler.sketch_manager.lock().unwrap();
        assert_eq!(mgr.get_sketches("1").len(), 1);
    }

    #[test]
    fn handle_command_with_other_replay_sessions_delivers_to_registered_sender() {
        use tokio::sync::mpsc;
        let handler = ServerCommandHandlerRemoveSketches::default();
        let (tx, mut rx) = mpsc::unbounded_channel();
        {
            let mut rsm = handler.replay_session_manager.lock().unwrap();
            rsm.add_session(1, "replay".to_string(), "Alice".to_string());
            rsm.add_session(2, "replay".to_string(), "Bob".to_string());
            rsm.register_sender(2, tx);
        }
        let command = ClientCommandRemoveSketches::with_ids(vec!["sk-1".to_string()]);
        assert!(handler.handle_command(1, &command));
        let sent = rx.try_recv().expect("expected a message forwarded to session 2");
        assert!(sent.contains("serverRemoveSketches"));
        assert!(sent.contains("sk-1"));
    }
}
