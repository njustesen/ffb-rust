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
    use crate::client_commands::{ClientCommand, ClientEndTurn};

    #[test]
    fn serialize_client_end_turn() {
        let cmd = ClientCommand::ClientEndTurn(ClientEndTurn);
        let json = serialize_client_command(&cmd).unwrap();
        assert!(json.contains("clientEndTurn"));
    }
}
