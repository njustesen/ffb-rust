/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerSketchSetLabel.
///
/// Java: `extends AbstractServerCommandHandlerSketch<ClientCommandSketchSetLabel, ServerCommandSketchSetLabel>`.
/// `AbstractServerCommandHandlerSketch` (this crate) is a generic composition helper
/// (`SketchCommandOps`), but its `handle_command` dispatch is re-implemented directly here
/// against this handler's own `sketch_manager` / `replay_session_manager`, matching the Java
/// source line for line — this mirrors how the sibling sketch handlers are structured.
use std::sync::{Arc, Mutex};

use ffb_engine::server_sketch_manager::ServerSketchManager;
use ffb_model::enums::NetCommandId;
use ffb_protocol::commands::client_command_sketch_set_label::ClientCommandSketchSetLabel;
use ffb_protocol::commands::server_command_sketch_set_label::ServerCommandSketchSetLabel;

use crate::model::received_command::SessionId;
use crate::net::replay_session_manager::ReplaySessionManager;

pub struct ServerCommandHandlerSketchSetLabel {
    /// Java: `sketchManager` (inherited from `AbstractServerCommandHandlerSketch`).
    sketch_manager: Arc<Mutex<ServerSketchManager>>,
    /// Java: `replaySessionManager` (inherited from `AbstractServerCommandHandlerSketch`).
    replay_session_manager: Arc<Mutex<ReplaySessionManager>>,
}

impl ServerCommandHandlerSketchSetLabel {
    pub fn new(
        sketch_manager: Arc<Mutex<ServerSketchManager>>,
        replay_session_manager: Arc<Mutex<ReplaySessionManager>>,
    ) -> Self {
        Self { sketch_manager, replay_session_manager }
    }

    /// Java: `getId()` — returns `NetCommandId.CLIENT_SKETCH_SET_LABEL`.
    pub fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientSketchSetLabel
    }

    /// Java: `updateSketchManager(Session, ClientCommandSketchSetLabel)` —
    /// `command.getSketchIds().forEach(id -> sketchManager.setLabel(session, id, command.getLabel()))`.
    pub fn update_sketch_manager(
        &self,
        session_id: SessionId,
        command: &ClientCommandSketchSetLabel,
    ) {
        let label = command.get_label().unwrap_or_default();
        let mut mgr = self.sketch_manager.lock().unwrap();
        for id in command.get_sketch_ids() {
            mgr.set_label(&session_id.to_string(), id, label);
        }
    }

    /// Java: `createServerCommand(String, ClientCommandSketchSetLabel)` —
    /// `new ServerCommandSketchSetLabel(coach, command.getSketchIds(), command.getLabel())`.
    pub fn create_server_command(
        &self,
        coach: &str,
        command: &ClientCommandSketchSetLabel,
    ) -> ServerCommandSketchSetLabel {
        ServerCommandSketchSetLabel::new(
            coach,
            command.get_sketch_ids().to_vec(),
            command.get_label().unwrap_or_default(),
        )
    }

    /// Java: `getServer().getCommunication().sendToReplaySession(otherSession, serverCommand)`'s
    /// wire payload. `ServerCommandSketchSetLabel` has no serde impl of its own (it isn't a
    /// `ffb_protocol::server_commands::ServerCommand` variant), so the JSON is built directly
    /// from its fields, same as `ServerCommandHandlerReplayLoaded`'s `ServerCommandStatus`.
    fn to_json(command: &ServerCommandSketchSetLabel) -> String {
        format!(
            "{{\"netCommandId\":\"{}\",\"coach\":{:?},\"sketchIds\":{},\"label\":{:?}}}",
            NetCommandId::ServerSketchSetLabel.name(),
            command.get_coach(),
            serde_json::to_string(command.get_sketch_ids()).unwrap_or_else(|_| "[]".to_string()),
            command.get_label()
        )
    }

    /// Java: `AbstractServerCommandHandlerSketch.handleCommand(ReceivedCommand)` (final, inherited).
    pub fn handle_command(
        &self,
        session_id: SessionId,
        command: &ClientCommandSketchSetLabel,
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

impl Default for ServerCommandHandlerSketchSetLabel {
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
        let _ = ServerCommandHandlerSketchSetLabel::default();
    }

    #[test]
    fn get_id_returns_client_sketch_set_label() {
        let handler = ServerCommandHandlerSketchSetLabel::default();
        assert_eq!(handler.get_id(), NetCommandId::ClientSketchSetLabel);
    }

    #[test]
    fn update_sketch_manager_sets_label_for_each_id() {
        let handler = ServerCommandHandlerSketchSetLabel::default();
        {
            let mut mgr = handler.sketch_manager.lock().unwrap();
            mgr.add_sketch("1", ManagerSketch::new("sk-1"));
            mgr.add_sketch("1", ManagerSketch::new("sk-2"));
        }
        let command = ClientCommandSketchSetLabel::with_label(
            vec!["sk-1".to_string(), "sk-2".to_string()],
            "Arrow",
        );
        handler.update_sketch_manager(1, &command);
        let mut mgr = handler.sketch_manager.lock().unwrap();
        assert_eq!(mgr.get_sketches("1").len(), 2);
    }

    #[test]
    fn create_server_command_carries_coach_ids_and_label() {
        let handler = ServerCommandHandlerSketchSetLabel::default();
        let command = ClientCommandSketchSetLabel::with_label(vec!["sk-1".to_string()], "Arrow");
        let server_command = handler.create_server_command("Alice", &command);
        assert_eq!(server_command.get_coach(), "Alice");
        assert_eq!(server_command.get_sketch_ids(), &["sk-1".to_string()]);
        assert_eq!(server_command.get_label(), "Arrow");
    }

    #[test]
    fn handle_command_without_replay_session_is_noop() {
        let handler = ServerCommandHandlerSketchSetLabel::default();
        let command = ClientCommandSketchSetLabel::with_label(vec!["sk-1".to_string()], "Arrow");
        assert!(handler.handle_command(1, &command));
    }

    #[test]
    fn handle_command_with_other_replay_sessions_delivers_to_registered_sender() {
        use tokio::sync::mpsc;
        let handler = ServerCommandHandlerSketchSetLabel::default();
        let (tx, mut rx) = mpsc::unbounded_channel();
        {
            let mut rsm = handler.replay_session_manager.lock().unwrap();
            rsm.add_session(1, "replay".to_string(), "Alice".to_string());
            rsm.add_session(2, "replay".to_string(), "Bob".to_string());
            rsm.register_sender(2, tx);
        }
        let command = ClientCommandSketchSetLabel::with_label(vec!["sk-1".to_string()], "Arrow");
        assert!(handler.handle_command(1, &command));
        let sent = rx.try_recv().expect("expected a message forwarded to session 2");
        assert!(sent.contains("serverSketchSetLabel"));
        assert!(sent.contains("Arrow"));
    }
}
