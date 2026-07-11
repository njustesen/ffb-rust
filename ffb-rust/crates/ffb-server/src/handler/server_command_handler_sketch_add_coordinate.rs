/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerSketchAddCoordinate.
///
/// Java: `extends AbstractServerCommandHandlerSketch<ClientCommandSketchAddCoordinate, ServerCommandSketchAddCoordinate>`.
/// `AbstractServerCommandHandlerSketch` (this crate) is a generic composition helper
/// (`SketchCommandOps`), but its `handle_command` dispatch is re-implemented directly here
/// against this handler's own `sketch_manager` / `replay_session_manager`, matching the Java
/// source line for line — this mirrors how the sibling sketch handlers are structured.
use std::sync::{Arc, Mutex};

use ffb_engine::server_sketch_manager::ServerSketchManager;
use ffb_model::enums::NetCommandId;
use ffb_model::types::FieldCoordinate;
use ffb_protocol::commands::client_command_sketch_add_coordinate::ClientCommandSketchAddCoordinate;
use ffb_protocol::commands::server_command_sketch_add_coordinate::ServerCommandSketchAddCoordinate;

use crate::model::received_command::SessionId;
use crate::net::replay_session_manager::ReplaySessionManager;

pub struct ServerCommandHandlerSketchAddCoordinate {
    /// Java: `sketchManager` (inherited from `AbstractServerCommandHandlerSketch`).
    sketch_manager: Arc<Mutex<ServerSketchManager>>,
    /// Java: `replaySessionManager` (inherited from `AbstractServerCommandHandlerSketch`).
    replay_session_manager: Arc<Mutex<ReplaySessionManager>>,
}

impl ServerCommandHandlerSketchAddCoordinate {
    pub fn new(
        sketch_manager: Arc<Mutex<ServerSketchManager>>,
        replay_session_manager: Arc<Mutex<ReplaySessionManager>>,
    ) -> Self {
        Self { sketch_manager, replay_session_manager }
    }

    /// Java: `getId()` — returns `NetCommandId.CLIENT_SKETCH_ADD_COORDINATE`.
    pub fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientSketchAddCoordinate
    }

    /// Java: `updateSketchManager(Session, ClientCommandSketchAddCoordinate)` —
    /// `sketchManager.addPathCoordinate(session, command.getSketchId(), command.getCoordinate())`.
    pub fn update_sketch_manager(
        &self,
        session_id: SessionId,
        command: &ClientCommandSketchAddCoordinate,
    ) {
        if let (Some(sketch_id), Some(coordinate)) =
            (command.get_sketch_id(), command.get_coordinate())
        {
            let mut mgr = self.sketch_manager.lock().unwrap();
            mgr.add_path_coordinate(&session_id.to_string(), sketch_id, coordinate.x, coordinate.y);
        }
    }

    /// Java: `createServerCommand(String, ClientCommandSketchAddCoordinate)` —
    /// `new ServerCommandSketchAddCoordinate(coach, command.getSketchId(), command.getCoordinate())`.
    pub fn create_server_command(
        &self,
        coach: &str,
        command: &ClientCommandSketchAddCoordinate,
    ) -> ServerCommandSketchAddCoordinate {
        ServerCommandSketchAddCoordinate::new(
            coach,
            command.get_sketch_id().unwrap_or_default(),
            command.get_coordinate().unwrap_or(FieldCoordinate::new(0, 0)),
        )
    }

    /// Java: `getServer().getCommunication().sendToReplaySession(otherSession, serverCommand)`'s
    /// wire payload. `ServerCommandSketchAddCoordinate` has no serde impl of its own (it isn't a
    /// `ffb_protocol::server_commands::ServerCommand` variant), so the JSON is built directly
    /// from its fields, same as `ServerCommandHandlerReplayLoaded`'s `ServerCommandStatus`.
    fn to_json(command: &ServerCommandSketchAddCoordinate) -> String {
        format!(
            "{{\"netCommandId\":\"{}\",\"coach\":{:?},\"sketchId\":{:?},\"coordinate\":{}}}",
            NetCommandId::ServerSketchAddCoordinate.name(),
            command.get_coach(),
            command.get_sketch_id(),
            serde_json::to_string(&command.get_coordinate()).unwrap_or_else(|_| "null".to_string())
        )
    }

    /// Java: `AbstractServerCommandHandlerSketch.handleCommand(ReceivedCommand)` (final, inherited).
    pub fn handle_command(
        &self,
        session_id: SessionId,
        command: &ClientCommandSketchAddCoordinate,
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

impl Default for ServerCommandHandlerSketchAddCoordinate {
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
        let _ = ServerCommandHandlerSketchAddCoordinate::default();
    }

    #[test]
    fn get_id_returns_client_sketch_add_coordinate() {
        let handler = ServerCommandHandlerSketchAddCoordinate::default();
        assert_eq!(handler.get_id(), NetCommandId::ClientSketchAddCoordinate);
    }

    #[test]
    fn update_sketch_manager_appends_coordinate() {
        let handler = ServerCommandHandlerSketchAddCoordinate::default();
        {
            let mut mgr = handler.sketch_manager.lock().unwrap();
            mgr.add_sketch("1", ManagerSketch::new("sk-1"));
        }
        let command =
            ClientCommandSketchAddCoordinate::with_sketch("sk-1", FieldCoordinate::new(3, 4));
        handler.update_sketch_manager(1, &command);
        let mut mgr = handler.sketch_manager.lock().unwrap();
        let sketches = mgr.get_sketches("1");
        assert_eq!(sketches.len(), 1);
    }

    #[test]
    fn update_sketch_manager_missing_fields_is_noop() {
        let handler = ServerCommandHandlerSketchAddCoordinate::default();
        handler.update_sketch_manager(1, &ClientCommandSketchAddCoordinate::new());
        let mut mgr = handler.sketch_manager.lock().unwrap();
        assert!(mgr.get_sketches("1").is_empty());
    }

    #[test]
    fn create_server_command_carries_coach_sketch_id_and_coordinate() {
        let handler = ServerCommandHandlerSketchAddCoordinate::default();
        let command =
            ClientCommandSketchAddCoordinate::with_sketch("sk-1", FieldCoordinate::new(5, 6));
        let server_command = handler.create_server_command("Alice", &command);
        assert_eq!(server_command.get_coach(), "Alice");
        assert_eq!(server_command.get_sketch_id(), "sk-1");
        assert_eq!(server_command.get_coordinate(), FieldCoordinate::new(5, 6));
    }

    #[test]
    fn handle_command_without_replay_session_is_noop() {
        let handler = ServerCommandHandlerSketchAddCoordinate::default();
        let command =
            ClientCommandSketchAddCoordinate::with_sketch("sk-1", FieldCoordinate::new(1, 1));
        assert!(handler.handle_command(1, &command));
        let mut mgr = handler.sketch_manager.lock().unwrap();
        assert!(mgr.get_sketches("1").is_empty());
    }

    #[test]
    fn handle_command_with_other_replay_sessions_delivers_to_registered_sender() {
        use tokio::sync::mpsc;
        let handler = ServerCommandHandlerSketchAddCoordinate::default();
        let (tx, mut rx) = mpsc::unbounded_channel();
        {
            let mut rsm = handler.replay_session_manager.lock().unwrap();
            rsm.add_session(1, "replay".to_string(), "Alice".to_string());
            rsm.add_session(2, "replay".to_string(), "Bob".to_string());
            rsm.register_sender(2, tx);
        }
        let command =
            ClientCommandSketchAddCoordinate::with_sketch("sk-1", FieldCoordinate::new(3, 4));
        assert!(handler.handle_command(1, &command));
        let sent = rx.try_recv().expect("expected a message forwarded to session 2");
        assert!(sent.contains("serverSketchAddCoordinate"));
        assert!(sent.contains("\"x\":3"));
    }
}
