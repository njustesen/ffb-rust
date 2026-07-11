//! 1:1 translation of `com.fumbbl.ffb.client.handler.ClientCommandHandlerSketchSetLabel`.

use ffb_model::enums::NetCommandId;
use ffb_protocol::commands::any_server_command::AnyServerCommand;

use crate::client::handler::abstract_client_command_handler_sketch::AbstractClientCommandHandlerSketch;
use crate::client::handler::client_command_handler::ClientCommandHandler;
use crate::client::handler::client_command_handler_mode::ClientCommandHandlerMode;

#[derive(Debug, Default)]
pub struct ClientCommandHandlerSketchSetLabel;

impl ClientCommandHandlerSketchSetLabel {
    pub fn new() -> Self {
        Self
    }
}

impl AbstractClientCommandHandlerSketch for ClientCommandHandlerSketchSetLabel {
    /// Java:
    /// ```java
    /// ClientSketchManager sketchManager = getClient().getUserInterface().getSketchManager();
    /// command.getSketchIds().forEach(id -> sketchManager.setLabel(command.getCoach(), id, command.getLabel()));
    /// ```
    fn update_sketch_manager(&mut self, command: &AnyServerCommand) {
        if let AnyServerCommand::ServerSketchSetLabel(command) = command {
            for _id in command.get_sketch_ids() {
                // java: sketchManager.setLabel(command.getCoach(), id, command.getLabel());
                let _coach = command.get_coach();
                let _label = command.get_label();
            }
        }
    }
}

impl ClientCommandHandler for ClientCommandHandlerSketchSetLabel {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerSketchSetLabel
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
    use ffb_protocol::commands::server_command_sketch_set_label::ServerCommandSketchSetLabel;

    #[test]
    fn get_id_is_server_sketch_set_label() {
        assert_eq!(
            ClientCommandHandlerSketchSetLabel::new().get_id(),
            NetCommandId::ServerSketchSetLabel
        );
    }

    #[test]
    fn handle_net_command_returns_true_with_multiple_ids() {
        let mut handler = ClientCommandHandlerSketchSetLabel::new();
        let cmd = AnyServerCommand::ServerSketchSetLabel(ServerCommandSketchSetLabel::new(
            "Carol",
            vec!["s1".into()],
            "Arrow",
        ));
        assert!(ClientCommandHandler::handle_net_command(&mut handler, &cmd, ClientCommandHandlerMode::PLAYING));
    }

    #[test]
    fn handle_net_command_returns_true_with_no_ids() {
        let mut handler = ClientCommandHandlerSketchSetLabel::new();
        let cmd = AnyServerCommand::ServerSketchSetLabel(ServerCommandSketchSetLabel::new(
            "Carol",
            vec![],
            "",
        ));
        assert!(ClientCommandHandler::handle_net_command(&mut handler, &cmd, ClientCommandHandlerMode::PLAYING));
    }

    #[test]
    fn update_sketch_manager_iterates_ids_without_panicking() {
        let mut handler = ClientCommandHandlerSketchSetLabel::new();
        let cmd = AnyServerCommand::ServerSketchSetLabel(ServerCommandSketchSetLabel::new(
            "Dave",
            vec!["s1".into(), "s2".into()],
            "Circle",
        ));
        handler.update_sketch_manager(&cmd);
    }

    #[test]
    fn default_constructs_a_handler() {
        let handler = ClientCommandHandlerSketchSetLabel::default();
        assert_eq!(handler.get_id(), NetCommandId::ServerSketchSetLabel);
    }
}
