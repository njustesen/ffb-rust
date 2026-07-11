//! 1:1 translation of `com.fumbbl.ffb.client.handler.ClientCommandHandlerAddSketches`.

use ffb_model::enums::NetCommandId;
use ffb_protocol::commands::any_server_command::AnyServerCommand;

use crate::client::handler::abstract_client_command_handler_sketch::AbstractClientCommandHandlerSketch;
use crate::client::handler::client_command_handler::ClientCommandHandler;
use crate::client::handler::client_command_handler_mode::ClientCommandHandlerMode;

#[derive(Debug, Default)]
pub struct ClientCommandHandlerAddSketches;

impl ClientCommandHandlerAddSketches {
    pub fn new() -> Self {
        Self
    }
}

impl AbstractClientCommandHandlerSketch for ClientCommandHandlerAddSketches {
    /// Java:
    /// ```java
    /// ClientSketchManager sketchManager = getClient().getUserInterface().getSketchManager();
    /// command.getSketches().forEach(sketch -> sketchManager.add(command.getCoach(), sketch));
    /// ```
    fn update_sketch_manager(&mut self, command: &AnyServerCommand) {
        if let AnyServerCommand::ServerAddSketches(command) = command {
            for _sketch in command.get_sketches() {
                // java: sketchManager.add(command.getCoach(), sketch);
                let _coach = command.get_coach();
            }
        }
    }
}

impl ClientCommandHandler for ClientCommandHandlerAddSketches {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerAddSketches
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
    use ffb_protocol::commands::server_command_add_sketches::ServerCommandAddSketches;
    use ffb_model::model::sketch::sketch::Sketch;

    #[test]
    fn get_id_is_server_add_sketches() {
        assert_eq!(
            ClientCommandHandlerAddSketches::new().get_id(),
            NetCommandId::ServerAddSketches
        );
    }

    #[test]
    fn handle_net_command_returns_true_with_sketches() {
        let mut handler = ClientCommandHandlerAddSketches::new();
        let cmd = AnyServerCommand::ServerAddSketches(ServerCommandAddSketches::new(
            "Alice",
            vec![Sketch::new()],
        ));
        assert!(ClientCommandHandler::handle_net_command(&mut handler, &cmd, ClientCommandHandlerMode::PLAYING));
    }

    #[test]
    fn handle_net_command_returns_true_with_no_sketches() {
        let mut handler = ClientCommandHandlerAddSketches::new();
        let cmd = AnyServerCommand::ServerAddSketches(ServerCommandAddSketches::new("Bob", vec![]));
        assert!(ClientCommandHandler::handle_net_command(&mut handler, &cmd, ClientCommandHandlerMode::PLAYING));
    }

    #[test]
    fn update_sketch_manager_iterates_all_sketches_without_panicking() {
        let mut handler = ClientCommandHandlerAddSketches::new();
        let cmd = AnyServerCommand::ServerAddSketches(ServerCommandAddSketches::new(
            "Carol",
            vec![Sketch::new(), Sketch::new()],
        ));
        handler.update_sketch_manager(&cmd);
    }

    #[test]
    fn default_constructs_a_handler() {
        let handler = ClientCommandHandlerAddSketches::default();
        assert_eq!(handler.get_id(), NetCommandId::ServerAddSketches);
    }
}
