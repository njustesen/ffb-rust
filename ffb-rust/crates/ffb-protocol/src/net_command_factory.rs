use crate::commands::{parse_server_command, ProtocolError};
use crate::server_commands::ServerCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.NetCommandFactory`.
/// Deserializes raw JSON payloads into typed `ServerCommand` values.
#[derive(Debug, Default)]
pub struct NetCommandFactory;

impl NetCommandFactory {
    pub fn new() -> Self {
        Self
    }

    /// Parse a raw JSON string from the server wire into a `ServerCommand`.
    /// Returns `None` if the input is null/empty.
    pub fn for_json_str(&self, json: &str) -> Result<Option<ServerCommand>, ProtocolError> {
        if json.trim().is_empty() || json.trim() == "null" {
            return Ok(None);
        }
        let cmd = parse_server_command(json)?;
        Ok(Some(cmd))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_json_returns_none() {
        let factory = NetCommandFactory::new();
        let result = factory.for_json_str("null").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn empty_string_returns_none() {
        let factory = NetCommandFactory::new();
        let result = factory.for_json_str("").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn invalid_json_returns_err() {
        let factory = NetCommandFactory::new();
        assert!(factory.for_json_str("{not valid}").is_err());
    }

    #[test]
    fn valid_pong_parses() {
        let factory = NetCommandFactory::new();
        let json = r#"{"netCommandId":"serverPong","timestamp":42}"#;
        let result = factory.for_json_str(json).unwrap();
        assert!(result.is_some());
    }
}
