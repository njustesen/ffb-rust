/// 1:1 translation of com.fumbbl.ffb.server.net.ReceivedCommand.
use ffb_protocol::client_commands::ClientCommand;
use crate::net::commands::any_internal_server_command::AnyInternalServerCommand;

/// Identifier for a connected WebSocket session.
pub type SessionId = u64;

/// Java: `ReceivedCommand.fCommand` is a `NetCommand`, which is either a `ClientCommand` or
/// an `InternalServerCommand` (Java: `isClientCommand()`/`isInternalCommand()`). This enum is
/// the Rust equivalent of that union — see `AnyInternalServerCommand`'s doc comment.
pub enum ReceivedNetCommand {
    Client(ClientCommand),
    Internal(AnyInternalServerCommand),
}

impl ReceivedNetCommand {
    /// Java: `ReceivedCommand.isClientCommand()`.
    pub fn is_client_command(&self) -> bool {
        matches!(self, ReceivedNetCommand::Client(_))
    }

    /// Java: `ReceivedCommand.isInternalCommand()`.
    pub fn is_internal_command(&self) -> bool {
        matches!(self, ReceivedNetCommand::Internal(_))
    }
}

/// A command received from a client session, including its session origin.
pub struct ReceivedCommand {
    /// Java: `fCommand`
    pub command: ReceivedNetCommand,
    /// Java: `fSession` (we use an opaque ID instead of a Jetty Session reference)
    pub session_id: SessionId,
}

impl ReceivedCommand {
    pub fn new(command: ClientCommand, session_id: SessionId) -> Self {
        Self { command: ReceivedNetCommand::Client(command), session_id }
    }

    /// Java: `new ReceivedCommand(InternalServerCommand, Session)`.
    pub fn new_internal(command: AnyInternalServerCommand, session_id: SessionId) -> Self {
        Self { command: ReceivedNetCommand::Internal(command), session_id }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_protocol::client_commands::{ClientCommand, ClientEndTurn};

    #[test]
    fn stores_command_and_session() {
        let cmd = ReceivedCommand::new(ClientCommand::ClientEndTurn(ClientEndTurn {}), 7);
        assert_eq!(cmd.session_id, 7);
    }

    #[test]
    fn client_command_is_client_not_internal() {
        let cmd = ReceivedCommand::new(ClientCommand::ClientEndTurn(ClientEndTurn {}), 1);
        assert!(cmd.command.is_client_command());
        assert!(!cmd.command.is_internal_command());
    }

    #[test]
    fn internal_command_is_internal_not_client() {
        use crate::net::commands::internal_server_command_close_game::InternalServerCommandCloseGame;
        let cmd = ReceivedCommand::new_internal(
            AnyInternalServerCommand::CloseGame(InternalServerCommandCloseGame::new(1)),
            2,
        );
        assert!(cmd.command.is_internal_command());
        assert!(!cmd.command.is_client_command());
    }
}
