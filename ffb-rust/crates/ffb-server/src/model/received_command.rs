/// 1:1 translation of com.fumbbl.ffb.server.net.ReceivedCommand.
use ffb_protocol::client_commands::ClientCommand;

/// Identifier for a connected WebSocket session.
pub type SessionId = u64;

/// A command received from a client session, including its session origin.
#[derive(Debug)]
pub struct ReceivedCommand {
    /// Java: `fCommand`
    pub command: ClientCommand,
    /// Java: `fSession` (we use an opaque ID instead of a Jetty Session reference)
    pub session_id: SessionId,
}

impl ReceivedCommand {
    pub fn new(command: ClientCommand, session_id: SessionId) -> Self {
        Self { command, session_id }
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
}
