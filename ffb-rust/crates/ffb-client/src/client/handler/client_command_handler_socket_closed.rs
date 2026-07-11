//! 1:1 translation of `com.fumbbl.ffb.client.handler.ClientCommandHandlerSocketClosed`.

use ffb_model::enums::NetCommandId;
use ffb_protocol::commands::any_server_command::AnyServerCommand;

use crate::client::handler::client_command_handler::ClientCommandHandler;
use crate::client::handler::client_command_handler_mode::ClientCommandHandlerMode;

#[derive(Debug, Default)]
pub struct ClientCommandHandlerSocketClosed;

impl ClientCommandHandlerSocketClosed {
    pub fn new() -> Self {
        Self
    }
}

impl ClientCommandHandler for ClientCommandHandlerSocketClosed {
    /// Java: `getId()`.
    fn get_id(&self) -> NetCommandId {
        NetCommandId::InternalServerSocketClosed
    }

    /// Java:
    /// ```java
    /// getClient().getUserInterface().socketClosed();
    /// getClient().logDebug("Connection closed by server.");
    /// return true;
    /// ```
    /// Both calls are GUI/client-logging side effects with no Rust equivalent
    /// yet (`FantasyFootballClient.rs`/`UserInterface.rs` are still stubs) —
    /// noted rather than invented.
    fn handle_net_command(
        &mut self,
        _net_command: &AnyServerCommand,
        _mode: ClientCommandHandlerMode,
    ) -> bool {
        // java: getClient().getUserInterface().socketClosed();
        // java: getClient().logDebug("Connection closed by server.");
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::SoundId;
    use ffb_protocol::commands::server_command_sound::ServerCommandSound;

    #[test]
    fn get_id_is_internal_server_socket_closed() {
        assert_eq!(
            ClientCommandHandlerSocketClosed::new().get_id(),
            NetCommandId::InternalServerSocketClosed
        );
    }

    #[test]
    fn handle_net_command_always_returns_true() {
        let mut handler = ClientCommandHandlerSocketClosed::new();
        let cmd = AnyServerCommand::ServerSound(ServerCommandSound::new(SoundId::TOUCHDOWN));
        assert!(handler.handle_net_command(&cmd, ClientCommandHandlerMode::PLAYING));
    }

    #[test]
    fn handle_net_command_returns_true_across_all_modes() {
        let mut handler = ClientCommandHandlerSocketClosed::new();
        let cmd = AnyServerCommand::ServerSound(ServerCommandSound::new(SoundId::TOUCHDOWN));
        assert!(handler.handle_net_command(&cmd, ClientCommandHandlerMode::REPLAYING));
        assert!(handler.handle_net_command(&cmd, ClientCommandHandlerMode::INITIALIZING));
        assert!(handler.handle_net_command(&cmd, ClientCommandHandlerMode::QUEUING));
    }

    #[test]
    fn default_constructs_a_handler() {
        let handler = ClientCommandHandlerSocketClosed::default();
        assert_eq!(handler.get_id(), NetCommandId::InternalServerSocketClosed);
    }
}
