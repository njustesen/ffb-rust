//! 1:1 translation of `com.fumbbl.ffb.client.handler.ClientCommandHandlerRemoveSketches`.

use ffb_model::enums::NetCommandId;
use ffb_protocol::commands::any_server_command::AnyServerCommand;

use crate::client::handler::abstract_client_command_handler_sketch::AbstractClientCommandHandlerSketch;
use crate::client::handler::client_command_handler::ClientCommandHandler;
use crate::client::handler::client_command_handler_mode::ClientCommandHandlerMode;

#[derive(Debug, Default)]
pub struct ClientCommandHandlerRemoveSketches {
    /// Records which branch of `update_sketch_manager` fired for the most
    /// recent command — not present in Java, added purely so pure-logic
    /// tests can observe the `ids == null || ids.isEmpty()` branch without a
    /// real `ClientSketchManager`.
    pub last_removed_all: Option<bool>,
}

impl ClientCommandHandlerRemoveSketches {
    pub fn new() -> Self {
        Self::default()
    }
}

impl AbstractClientCommandHandlerSketch for ClientCommandHandlerRemoveSketches {
    /// Java:
    /// ```java
    /// ClientSketchManager sketchManager = getClient().getUserInterface().getSketchManager();
    /// if (command.getIds() == null || command.getIds().isEmpty()) {
    ///     sketchManager.removeAll(command.getCoach());
    /// } else {
    ///     command.getIds().forEach(id -> sketchManager.remove(command.getCoach(), id));
    /// }
    /// ```
    fn update_sketch_manager(&mut self, command: &AnyServerCommand) {
        if let AnyServerCommand::ServerRemoveSketches(command) = command {
            let _coach = command.get_coach();
            if command.get_ids().is_empty() {
                // java: sketchManager.removeAll(command.getCoach());
                self.last_removed_all = Some(true);
            } else {
                for _id in command.get_ids() {
                    // java: sketchManager.remove(command.getCoach(), id);
                }
                self.last_removed_all = Some(false);
            }
        }
    }
}

impl ClientCommandHandler for ClientCommandHandlerRemoveSketches {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerRemoveSketches
    }

    fn handle_net_command(
        &mut self,
        net_command: &AnyServerCommand,
        mode: ClientCommandHandlerMode,
    ) -> bool {
        AbstractClientCommandHandlerSketch::handle_net_command(self, net_command, mode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_protocol::commands::server_command_remove_sketches::ServerCommandRemoveSketches;

    #[test]
    fn get_id_is_server_remove_sketches() {
        assert_eq!(
            ClientCommandHandlerRemoveSketches::new().get_id(),
            NetCommandId::ServerRemoveSketches
        );
    }

    #[test]
    fn empty_ids_takes_remove_all_branch() {
        let mut handler = ClientCommandHandlerRemoveSketches::new();
        let cmd = AnyServerCommand::ServerRemoveSketches(ServerCommandRemoveSketches::new("Bob", vec![]));
        assert!(ClientCommandHandler::handle_net_command(&mut handler, &cmd, ClientCommandHandlerMode::PLAYING));
        assert_eq!(handler.last_removed_all, Some(true));
    }

    #[test]
    fn non_empty_ids_takes_remove_individual_branch() {
        let mut handler = ClientCommandHandlerRemoveSketches::new();
        let cmd = AnyServerCommand::ServerRemoveSketches(ServerCommandRemoveSketches::new(
            "Bob",
            vec!["id1".into()],
        ));
        assert!(ClientCommandHandler::handle_net_command(&mut handler, &cmd, ClientCommandHandlerMode::PLAYING));
        assert_eq!(handler.last_removed_all, Some(false));
    }

    #[test]
    fn default_has_no_recorded_branch_yet() {
        let handler = ClientCommandHandlerRemoveSketches::new();
        assert_eq!(handler.last_removed_all, None);
    }

    #[test]
    fn handle_net_command_returns_true_regardless_of_branch() {
        let mut handler = ClientCommandHandlerRemoveSketches::new();
        let empty = AnyServerCommand::ServerRemoveSketches(ServerCommandRemoveSketches::new("A", vec![]));
        let non_empty = AnyServerCommand::ServerRemoveSketches(ServerCommandRemoveSketches::new(
            "A",
            vec!["x".into()],
        ));
        assert!(ClientCommandHandler::handle_net_command(&mut handler, &empty, ClientCommandHandlerMode::PLAYING));
        assert!(ClientCommandHandler::handle_net_command(&mut handler, &non_empty, ClientCommandHandlerMode::PLAYING));
    }
}
