use ffb_model::enums::NetCommandId;
use crate::commands::any_client_command::AnyClientCommand;
use crate::commands::any_server_command::AnyServerCommand;
use crate::commands::{parse_server_command, ProtocolError};
use crate::server_commands::ServerCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.NetCommandFactory`.
/// Deserializes raw JSON payloads into typed `ServerCommand` values.
#[derive(Debug, Default)]
pub struct NetCommandFactory;

/// Result of the real `forJsonValue` dispatch: Java's `NetCommand` is a single
/// polymorphic base class covering both directions; Rust splits client/server
/// into two sum types (`AnyClientCommand`/`AnyServerCommand`), so this wraps
/// whichever one `NetCommandId.createNetCommand()` would have produced.
#[derive(Debug)]
pub enum AnyNetCommand {
    Client(AnyClientCommand),
    Server(AnyServerCommand),
}

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

    /// Java: `NetCommandFactory.forJsonValue(source, jsonValue)` — reads the
    /// `netCommandId` key, looks it up via `NetCommandId.createNetCommand()`
    /// (here: `AnyClientCommand`/`AnyServerCommand::from_json`), then calls
    /// the concrete command's `initFrom`. Dispatches against the 123 genuine
    /// 1:1-translated `commands::` structs, not the hand-rolled
    /// `client_commands`/`server_commands` simplification `for_json_str` uses.
    pub fn for_json_value(&self, json: &serde_json::Value) -> Option<AnyNetCommand> {
        if json.is_null() {
            return None;
        }
        let id = NetCommandId::from_name(json.get("netCommandId")?.as_str()?)?;
        if let Some(cmd) = AnyClientCommand::from_json(id, json) {
            return Some(AnyNetCommand::Client(cmd));
        }
        if let Some(cmd) = AnyServerCommand::from_json(id, json) {
            return Some(AnyNetCommand::Server(cmd));
        }
        None
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

    #[test]
    fn for_json_value_null_returns_none() {
        let factory = NetCommandFactory::new();
        assert!(factory.for_json_value(&serde_json::Value::Null).is_none());
    }

    #[test]
    fn for_json_value_missing_net_command_id_returns_none() {
        let factory = NetCommandFactory::new();
        assert!(factory.for_json_value(&serde_json::json!({})).is_none());
    }

    #[test]
    fn for_json_value_dispatches_client_command() {
        let factory = NetCommandFactory::new();
        let json = serde_json::json!({"netCommandId": "clientJoin"});
        let result = factory.for_json_value(&json).unwrap();
        assert!(matches!(result, AnyNetCommand::Client(AnyClientCommand::ClientJoin(_))));
    }

    #[test]
    fn for_json_value_dispatches_server_command() {
        let factory = NetCommandFactory::new();
        let json = serde_json::json!({"netCommandId": "serverGameTime", "gameTime": 1, "turnTime": 2});
        let result = factory.for_json_value(&json).unwrap();
        assert!(matches!(result, AnyNetCommand::Server(AnyServerCommand::ServerGameTime(_))));
    }
}
