/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerAddSketch.
///
/// Java: `extends AbstractServerCommandHandlerSketch<ClientCommandAddSketch, ServerCommandAddSketches>`.
/// `AbstractServerCommandHandlerSketch` (this crate) is a generic composition helper
/// (`SketchCommandOps`), but its `handle_command` dispatch is re-implemented directly here
/// against this handler's own `sketch_manager` / `replay_session_manager`, matching the Java
/// source line for line — this mirrors how the sibling sketch handlers are structured.
use std::sync::{Arc, Mutex};

use ffb_engine::server_sketch_manager::{ServerSketchManager, Sketch as ManagerSketch};
use ffb_model::enums::NetCommandId;
use ffb_model::model::sketch::sketch::Sketch as ModelSketch;
use ffb_protocol::commands::client_command_add_sketch::ClientCommandAddSketch;
use ffb_protocol::commands::server_command_add_sketches::ServerCommandAddSketches;

use crate::model::received_command::SessionId;
use crate::net::replay_session_manager::ReplaySessionManager;

pub struct ServerCommandHandlerAddSketch {
    /// Java: `sketchManager` (inherited from `AbstractServerCommandHandlerSketch`).
    sketch_manager: Arc<Mutex<ServerSketchManager>>,
    /// Java: `replaySessionManager` (inherited from `AbstractServerCommandHandlerSketch`).
    replay_session_manager: Arc<Mutex<ReplaySessionManager>>,
}

impl ServerCommandHandlerAddSketch {
    pub fn new(
        sketch_manager: Arc<Mutex<ServerSketchManager>>,
        replay_session_manager: Arc<Mutex<ReplaySessionManager>>,
    ) -> Self {
        Self { sketch_manager, replay_session_manager }
    }

    /// Java: `getId()` — returns `NetCommandId.CLIENT_ADD_SKETCH`.
    pub fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientAddSketch
    }

    /// Java: `updateSketchManager(Session, ClientCommandAddSketch)` —
    /// `sketchManager.addSketch(session, command.getSketch())`.
    pub fn update_sketch_manager(&self, session_id: SessionId, command: &ClientCommandAddSketch) {
        if let Some(sketch) = command.get_sketch() {
            let mut manager_sketch = ManagerSketch::new(sketch.get_id());
            manager_sketch.set_rgb(sketch.get_rgb());
            if let Some(label) = sketch.get_label() {
                manager_sketch.set_label(label);
            }
            for coordinate in sketch.get_path() {
                manager_sketch.add_coordinate(coordinate.x, coordinate.y);
            }
            let mut mgr = self.sketch_manager.lock().unwrap();
            mgr.add_sketch(&session_id.to_string(), manager_sketch);
        }
    }

    /// Java: `createServerCommand(String, ClientCommandAddSketch)` —
    /// `new ServerCommandAddSketches(coach, Collections.singletonList(command.getSketch()))`.
    pub fn create_server_command(
        &self,
        coach: &str,
        command: &ClientCommandAddSketch,
    ) -> ServerCommandAddSketches {
        let sketch = command.get_sketch().cloned().unwrap_or_default();
        let mut model_sketch = ModelSketch::new();
        model_sketch.id = sketch.get_id().to_string();
        model_sketch.set_rgb(sketch.get_rgb());
        if let Some(label) = sketch.get_label() {
            model_sketch.set_label(label);
        }
        for coordinate in sketch.get_path() {
            model_sketch.add_position(*coordinate);
        }
        ServerCommandAddSketches::new(coach, vec![model_sketch])
    }

    /// Java: `getServer().getCommunication().sendToReplaySession(otherSession, serverCommand)`'s
    /// wire payload. `ServerCommandAddSketches` has no serde impl of its own (it isn't a
    /// `ffb_protocol::server_commands::ServerCommand` variant), so the JSON is built directly
    /// from its fields, same as `ServerCommandHandlerReplayLoaded`'s `ServerCommandStatus`.
    fn to_json(coach: &str, command: &ServerCommandAddSketches) -> String {
        format!(
            "{{\"netCommandId\":\"{}\",\"coach\":{:?},\"sketches\":{}}}",
            NetCommandId::ServerAddSketches.name(),
            coach,
            serde_json::to_string(command.get_sketches()).unwrap_or_else(|_| "[]".to_string())
        )
    }

    /// Java: `AbstractServerCommandHandlerSketch.handleCommand(ReceivedCommand)` (final, inherited).
    pub fn handle_command(&self, session_id: SessionId, command: &ClientCommandAddSketch) -> bool {
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
                let json = Self::to_json(&coach, &server_command);
                self.replay_session_manager.lock().unwrap().send_to(other_session, &json);
            }
        }
        true
    }
}

impl Default for ServerCommandHandlerAddSketch {
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

    #[test]
    fn construct() {
        let _ = ServerCommandHandlerAddSketch::default();
    }

    #[test]
    fn get_id_returns_client_add_sketch() {
        let handler = ServerCommandHandlerAddSketch::default();
        assert_eq!(handler.get_id(), NetCommandId::ClientAddSketch);
    }

    #[test]
    fn update_sketch_manager_adds_sketch_from_id() {
        let handler = ServerCommandHandlerAddSketch::default();
        let command = ClientCommandAddSketch::with_sketch_id("sk-1");
        handler.update_sketch_manager(7, &command);
        let mut mgr = handler.sketch_manager.lock().unwrap();
        let sketches = mgr.get_sketches("7");
        assert_eq!(sketches.len(), 1);
        assert_eq!(sketches[0].get_id(), "sk-1");
    }

    #[test]
    fn update_sketch_manager_no_id_is_noop() {
        let handler = ServerCommandHandlerAddSketch::default();
        let command = ClientCommandAddSketch::new();
        handler.update_sketch_manager(7, &command);
        let mut mgr = handler.sketch_manager.lock().unwrap();
        assert!(mgr.get_sketches("7").is_empty());
    }

    #[test]
    fn create_server_command_carries_coach_and_one_sketch() {
        let handler = ServerCommandHandlerAddSketch::default();
        let command = ClientCommandAddSketch::with_sketch_id("sk-1");
        let server_command = handler.create_server_command("Alice", &command);
        assert_eq!(server_command.get_sketches().len(), 1);
    }

    #[test]
    fn handle_command_without_replay_session_is_noop() {
        let handler = ServerCommandHandlerAddSketch::default();
        let command = ClientCommandAddSketch::with_sketch_id("sk-1");
        assert!(handler.handle_command(1, &command));
        let mut mgr = handler.sketch_manager.lock().unwrap();
        assert!(mgr.get_sketches("1").is_empty());
    }

    #[test]
    fn handle_command_with_replay_session_and_no_others_updates_manager() {
        let handler = ServerCommandHandlerAddSketch::default();
        {
            let mut rsm = handler.replay_session_manager.lock().unwrap();
            rsm.add_session(1, "replay".to_string(), "Alice".to_string());
        }
        let command = ClientCommandAddSketch::with_sketch_id("sk-1");
        assert!(handler.handle_command(1, &command));
        let mut mgr = handler.sketch_manager.lock().unwrap();
        assert_eq!(mgr.get_sketches("1").len(), 1);
    }

    #[test]
    fn handle_command_with_other_replay_sessions_delivers_to_registered_sender() {
        use tokio::sync::mpsc;
        let handler = ServerCommandHandlerAddSketch::default();
        let (tx, mut rx) = mpsc::unbounded_channel();
        {
            let mut rsm = handler.replay_session_manager.lock().unwrap();
            rsm.add_session(1, "replay".to_string(), "Alice".to_string());
            rsm.add_session(2, "replay".to_string(), "Bob".to_string());
            rsm.register_sender(2, tx);
        }
        let command = ClientCommandAddSketch::with_sketch_id("sk-1");
        assert!(handler.handle_command(1, &command));
        let sent = rx.try_recv().expect("expected a message forwarded to session 2");
        assert!(sent.contains("serverAddSketches"));
        assert!(sent.contains("Alice"));
    }
}
