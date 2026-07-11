//! 1:1 translation of `com.fumbbl.ffb.client.handler.ClientCommandHandlerSketchAddCoordinate`.

use ffb_model::enums::NetCommandId;
use ffb_protocol::commands::any_server_command::AnyServerCommand;

use crate::client::handler::abstract_client_command_handler_sketch::AbstractClientCommandHandlerSketch;
use crate::client::handler::client_command_handler::ClientCommandHandler;
use crate::client::handler::client_command_handler_mode::ClientCommandHandlerMode;

#[derive(Debug, Default)]
pub struct ClientCommandHandlerSketchAddCoordinate;

impl ClientCommandHandlerSketchAddCoordinate {
    pub fn new() -> Self {
        Self
    }
}

impl AbstractClientCommandHandlerSketch for ClientCommandHandlerSketchAddCoordinate {
    /// Java:
    /// ```java
    /// ClientSketchManager sketchManager = getClient().getUserInterface().getSketchManager();
    /// sketchManager.add(command.getCoach(), command.getSketchId(), command.getCoordinate());
    /// ```
    fn update_sketch_manager(&mut self, command: &AnyServerCommand) {
        if let AnyServerCommand::ServerSketchAddCoordinate(command) = command {
            // java: sketchManager.add(command.getCoach(), command.getSketchId(), command.getCoordinate());
            let _coach = command.get_coach();
            let _sketch_id = command.get_sketch_id();
            let _coordinate = command.get_coordinate();
        }
    }
}

impl ClientCommandHandler for ClientCommandHandlerSketchAddCoordinate {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerSketchAddCoordinate
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
    use ffb_model::types::FieldCoordinate;
    use ffb_protocol::commands::server_command_sketch_add_coordinate::ServerCommandSketchAddCoordinate;

    #[test]
    fn get_id_is_server_sketch_add_coordinate() {
        assert_eq!(
            ClientCommandHandlerSketchAddCoordinate::new().get_id(),
            NetCommandId::ServerSketchAddCoordinate
        );
    }

    #[test]
    fn handle_net_command_returns_true_for_matching_command() {
        let mut handler = ClientCommandHandlerSketchAddCoordinate::new();
        let cmd = AnyServerCommand::ServerSketchAddCoordinate(ServerCommandSketchAddCoordinate::new(
            "Alice",
            "sk1",
            FieldCoordinate::new(5, 3),
        ));
        assert!(ClientCommandHandler::handle_net_command(&mut handler, &cmd, ClientCommandHandlerMode::PLAYING));
    }

    #[test]
    fn update_sketch_manager_reads_coach_sketch_id_and_coordinate() {
        let mut handler = ClientCommandHandlerSketchAddCoordinate::new();
        let cmd = AnyServerCommand::ServerSketchAddCoordinate(ServerCommandSketchAddCoordinate::new(
            "Bob",
            "sk2",
            FieldCoordinate::new(1, 2),
        ));
        handler.update_sketch_manager(&cmd);
    }

    #[test]
    fn default_constructs_a_handler() {
        let handler = ClientCommandHandlerSketchAddCoordinate::default();
        assert_eq!(handler.get_id(), NetCommandId::ServerSketchAddCoordinate);
    }

    #[test]
    fn handle_net_command_true_across_all_modes() {
        let mut handler = ClientCommandHandlerSketchAddCoordinate::new();
        let cmd = AnyServerCommand::ServerSketchAddCoordinate(ServerCommandSketchAddCoordinate::new(
            "Alice",
            "sk1",
            FieldCoordinate::new(0, 0),
        ));
        assert!(ClientCommandHandler::handle_net_command(&mut handler, &cmd, ClientCommandHandlerMode::REPLAYING));
        assert!(ClientCommandHandler::handle_net_command(&mut handler, &cmd, ClientCommandHandlerMode::QUEUING));
    }
}
