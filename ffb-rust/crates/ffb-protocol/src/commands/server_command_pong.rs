use ffb_model::enums::NetCommandId;
use ffb_model::model::factory_type::FactoryContext;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandPong`.
/// Server-to-client heartbeat pong response.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandPong {
    /// Java: base-class `ServerCommand.fCommandNr`.
    pub command_nr: i32,
    /// Java: `fTimestamp` — echoed client timestamp.
    pub timestamp: i64,
}

impl ServerCommandPong {
    pub fn new(timestamp: i64) -> Self { Self { command_nr: 0, timestamp } }
    pub fn get_timestamp(&self) -> i64 { self.timestamp }

    /// Java: `ServerCommandPong.toJsonValue()` — writes `netCommandId` +
    /// `timestamp` only, no `commandNr` (unlike most `ServerCommand`
    /// subclasses).
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "netCommandId": self.get_id().name(),
            "timestamp": self.timestamp,
        })
    }

    /// Java: `ServerCommandPong.initFrom(source, jsonValue)` — no `commandNr`
    /// on the wire, so it's never parsed.
    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            command_nr: 0,
            timestamp: json.get("timestamp").and_then(|v| v.as_i64()).unwrap_or(0),
        }
    }
}

impl NetCommand for ServerCommandPong {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerPong
    }

    /// Java: `ServerCommandPong.getContext()` returns `FactoryContext.APPLICATION`.
    fn get_context(&self) -> FactoryContext {
        FactoryContext::APPLICATION
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamp_stored() {
        let cmd = ServerCommandPong::new(99999);
        assert_eq!(cmd.get_timestamp(), 99999);
    }

    #[test]
    fn default_zero() {
        let cmd = ServerCommandPong::default();
        assert_eq!(cmd.timestamp, 0);
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ServerCommandPong::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ServerCommandPong::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ServerCommandPong::default());
        assert!(s.contains("ServerCommandPong"));
    }

    #[test]
    fn get_id_is_server_pong() {
        assert_eq!(ServerCommandPong::new(1).get_id(), NetCommandId::ServerPong);
    }

    #[test]
    fn context_is_application() {
        assert_eq!(ServerCommandPong::new(1).get_context(), FactoryContext::APPLICATION);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_timestamp() {
        let cmd = ServerCommandPong::new(555);
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "serverPong");
        assert_eq!(json["timestamp"], 555);
    }

    #[test]
    fn to_json_value_never_includes_command_nr() {
        let cmd = ServerCommandPong::new(1);
        let json = cmd.to_json_value();
        assert!(json.get("commandNr").is_none());
    }

    #[test]
    fn round_trip_with_timestamp() {
        let cmd = ServerCommandPong::new(9876);
        let json = cmd.to_json_value();
        let restored = ServerCommandPong::from_json(&json);
        assert_eq!(restored.timestamp, 9876);
    }

    #[test]
    fn round_trip_with_default_zero_timestamp() {
        let cmd = ServerCommandPong::default();
        let json = cmd.to_json_value();
        let restored = ServerCommandPong::from_json(&json);
        assert_eq!(restored.timestamp, 0);
    }
}
