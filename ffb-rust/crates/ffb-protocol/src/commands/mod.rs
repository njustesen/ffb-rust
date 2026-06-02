use thiserror::Error;
use crate::client_commands::ClientCommand;
use crate::server_commands::ServerCommand;

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unknown command id: {0}")]
    UnknownCommand(String),
}

/// Parse a raw JSON payload from the server into a `ServerCommand`.
pub fn parse_server_command(json: &str) -> Result<ServerCommand, ProtocolError> {
    Ok(serde_json::from_str(json)?)
}

/// Serialize a `ClientCommand` to JSON for sending to the server.
pub fn serialize_client_command(cmd: &ClientCommand) -> Result<String, ProtocolError> {
    Ok(serde_json::to_string(cmd)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client_commands::{ClientCommand, ClientEndTurn, ClientBlock};
    use crate::server_commands::{ServerCommand, ServerPong};

    #[test]
    fn serialize_client_end_turn() {
        let cmd = ClientCommand::ClientEndTurn(ClientEndTurn);
        let json = serialize_client_command(&cmd).unwrap();
        assert!(json.contains("clientEndTurn"));
    }

    #[test]
    fn serialize_then_parse_server_pong() {
        let cmd = ServerCommand::ServerPong(ServerPong { timestamp: 9999 });
        let json = serde_json::to_string(&cmd).unwrap();
        let back = parse_server_command(&json).unwrap();
        assert!(matches!(back, ServerCommand::ServerPong(ServerPong { timestamp: 9999 })));
    }

    #[test]
    fn parse_server_command_returns_error_on_bad_json() {
        let result = parse_server_command("{not valid json}");
        assert!(result.is_err(), "invalid JSON must return Err");
    }

    #[test]
    fn serialize_client_block() {
        let cmd = ClientCommand::ClientBlock(ClientBlock { defender_id: "p7".into() });
        let json = serialize_client_command(&cmd).unwrap();
        assert!(json.contains("clientBlock"), "must contain command tag");
        assert!(json.contains("p7"), "must contain defender_id");
    }
}
