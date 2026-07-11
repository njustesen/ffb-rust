//! 1:1 translation of `com.fumbbl.ffb.client.handler.ClientCommandHandlerClearSketches`.

use ffb_model::enums::NetCommandId;
use ffb_protocol::commands::any_server_command::AnyServerCommand;

use crate::client::handler::abstract_client_command_handler_sketch::AbstractClientCommandHandlerSketch;
use crate::client::handler::client_command_handler::ClientCommandHandler;
use crate::client::handler::client_command_handler_mode::ClientCommandHandlerMode;

#[derive(Debug, Default)]
pub struct ClientCommandHandlerClearSketches;

impl ClientCommandHandlerClearSketches {
    pub fn new() -> Self {
        Self
    }
}

impl AbstractClientCommandHandlerSketch for ClientCommandHandlerClearSketches {
    /// Java: `updateSketchManager(ServerCommandClearSketches command)`.
    fn update_sketch_manager(&mut self, command: &AnyServerCommand) {
        if let AnyServerCommand::ServerClearSketches(_command) = command {
            // java: getClient().getUserInterface().getSketchManager().clearAll();
        }
    }
}

impl ClientCommandHandler for ClientCommandHandlerClearSketches {
    /// Java: `getId()`.
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerClearSketches
    }

    /// Java: inherited `final handleNetCommand(...)` from `AbstractClientCommandHandlerSketch`.
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
    use ffb_protocol::commands::server_command_clear_sketches::ServerCommandClearSketches;
    use ffb_protocol::commands::server_command_sound::ServerCommandSound;
    use ffb_model::model::SoundId;

    #[test]
    fn get_id_is_server_clear_sketches() {
        assert_eq!(
            ClientCommandHandlerClearSketches::new().get_id(),
            NetCommandId::ServerClearSketches
        );
    }

    #[test]
    fn handle_net_command_returns_true_for_matching_command() {
        let mut handler = ClientCommandHandlerClearSketches::new();
        let cmd = AnyServerCommand::ServerClearSketches(ServerCommandClearSketches::new());
        assert!(ClientCommandHandler::handle_net_command(&mut handler, &cmd, ClientCommandHandlerMode::PLAYING));
    }

    #[test]
    fn handle_net_command_returns_true_across_modes() {
        let mut handler = ClientCommandHandlerClearSketches::new();
        let cmd = AnyServerCommand::ServerClearSketches(ServerCommandClearSketches::new());
        assert!(ClientCommandHandler::handle_net_command(&mut handler, &cmd, ClientCommandHandlerMode::REPLAYING));
        assert!(ClientCommandHandler::handle_net_command(&mut handler, &cmd, ClientCommandHandlerMode::INITIALIZING));
    }

    #[test]
    fn update_sketch_manager_ignores_mismatched_command_type() {
        // Not part of Java (which force-casts), but documents that the Rust
        // variant match is a no-op — not a panic — for a wrong command type.
        let mut handler = ClientCommandHandlerClearSketches::new();
        let cmd = AnyServerCommand::ServerSound(ServerCommandSound::new(SoundId::TOUCHDOWN));
        handler.update_sketch_manager(&cmd);
    }

    #[test]
    fn default_constructs_a_handler() {
        let handler = ClientCommandHandlerClearSketches::default();
        assert_eq!(handler.get_id(), NetCommandId::ServerClearSketches);
    }
}
