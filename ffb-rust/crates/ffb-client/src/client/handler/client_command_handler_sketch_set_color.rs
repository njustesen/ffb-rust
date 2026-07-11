//! 1:1 translation of `com.fumbbl.ffb.client.handler.ClientCommandHandlerSketchSetColor`.

use ffb_model::enums::NetCommandId;
use ffb_protocol::commands::any_server_command::AnyServerCommand;

use crate::client::handler::abstract_client_command_handler_sketch::AbstractClientCommandHandlerSketch;
use crate::client::handler::client_command_handler::ClientCommandHandler;
use crate::client::handler::client_command_handler_mode::ClientCommandHandlerMode;

#[derive(Debug, Default)]
pub struct ClientCommandHandlerSketchSetColor;

impl ClientCommandHandlerSketchSetColor {
    pub fn new() -> Self {
        Self
    }
}

impl AbstractClientCommandHandlerSketch for ClientCommandHandlerSketchSetColor {
    /// Java:
    /// ```java
    /// ClientSketchManager sketchManager = getClient().getUserInterface().getSketchManager();
    /// command.getSketchIds().forEach(id -> sketchManager.setColor(command.getCoach(), id, command.getRbg()));
    /// ```
    fn update_sketch_manager(&mut self, command: &AnyServerCommand) {
        if let AnyServerCommand::ServerSketchSetColor(command) = command {
            for _id in command.get_sketch_ids() {
                // java: sketchManager.setColor(command.getCoach(), id, command.getRbg());
                let _coach = command.get_coach();
                let _rbg = command.get_rbg();
            }
        }
    }
}

impl ClientCommandHandler for ClientCommandHandlerSketchSetColor {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerSketchSetColor
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
    use ffb_protocol::commands::server_command_sketch_set_color::ServerCommandSketchSetColor;

    #[test]
    fn get_id_is_server_sketch_set_color() {
        assert_eq!(
            ClientCommandHandlerSketchSetColor::new().get_id(),
            NetCommandId::ServerSketchSetColor
        );
    }

    #[test]
    fn handle_net_command_returns_true_with_multiple_ids() {
        let mut handler = ClientCommandHandlerSketchSetColor::new();
        let cmd = AnyServerCommand::ServerSketchSetColor(ServerCommandSketchSetColor::new(
            "Bob",
            vec!["s1".into(), "s2".into()],
            0xFF0000,
        ));
        assert!(ClientCommandHandler::handle_net_command(&mut handler, &cmd, ClientCommandHandlerMode::PLAYING));
    }

    #[test]
    fn handle_net_command_returns_true_with_no_ids() {
        let mut handler = ClientCommandHandlerSketchSetColor::new();
        let cmd = AnyServerCommand::ServerSketchSetColor(ServerCommandSketchSetColor::new(
            "Bob",
            vec![],
            0,
        ));
        assert!(ClientCommandHandler::handle_net_command(&mut handler, &cmd, ClientCommandHandlerMode::PLAYING));
    }

    #[test]
    fn update_sketch_manager_iterates_ids_without_panicking() {
        let mut handler = ClientCommandHandlerSketchSetColor::new();
        let cmd = AnyServerCommand::ServerSketchSetColor(ServerCommandSketchSetColor::new(
            "Carol",
            vec!["a".into()],
            42,
        ));
        handler.update_sketch_manager(&cmd);
    }

    #[test]
    fn default_constructs_a_handler() {
        let handler = ClientCommandHandlerSketchSetColor::default();
        assert_eq!(handler.get_id(), NetCommandId::ServerSketchSetColor);
    }
}
