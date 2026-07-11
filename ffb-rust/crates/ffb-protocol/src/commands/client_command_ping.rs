use ffb_model::enums::NetCommandId;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandPing`.
/// Heartbeat ping from client to server.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandPing {
    /// Java: base-class `ClientCommand.fEntropy`. Note: `ClientCommandPing`
    /// overrides `toJsonValue()`/`initFrom()` without calling `super`, so
    /// entropy is never part of the wire format for this command.
    pub entropy: Option<u8>,
    /// Java: `fTimestamp` — client-side timestamp at ping send time.
    pub timestamp: i64,
}

impl ClientCommandPing {
    pub fn new(timestamp: i64) -> Self { Self { entropy: None, timestamp } }
    pub fn get_timestamp(&self) -> i64 { self.timestamp }

    /// Java: `ClientCommandPing.toJsonValue()` — builds a fresh `JsonObject`
    /// rather than calling `super.toJsonValue()`, so entropy is not included.
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "netCommandId": self.get_id().name(),
            "timestamp": self.timestamp,
        })
    }

    /// Java: `ClientCommandPing.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            entropy: None,
            timestamp: json.get("timestamp").and_then(|v| v.as_i64()).unwrap_or(0),
        }
    }
}

impl NetCommand for ClientCommandPing {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientPing
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn timestamp_stored() {
        let cmd = ClientCommandPing::new(12345);
        assert_eq!(cmd.get_timestamp(), 12345);
    }
    #[test]
    fn default_zero() {
        let cmd = ClientCommandPing::default();
        assert_eq!(cmd.timestamp, 0);
    }
#[test]    fn debug_format_nonempty() {        let v = ClientCommandPing::default();        assert!(!format!("{:?}", v).is_empty());    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandPing::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandPing::default());
        assert!(s.contains("ClientCommandPing"));
    }

    #[test]
    fn get_id_is_client_ping() {
        assert_eq!(ClientCommandPing::default().get_id(), NetCommandId::ClientPing);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_timestamp() {
        let cmd = ClientCommandPing::new(999);
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientPing");
        assert_eq!(json["timestamp"], 999);
    }

    #[test]
    fn round_trip_with_data() {
        let cmd = ClientCommandPing::new(42);
        let json = cmd.to_json_value();
        let restored = ClientCommandPing::from_json(&json);
        assert_eq!(restored.get_timestamp(), 42);
    }

    #[test]
    fn round_trip_default() {
        let cmd = ClientCommandPing::default();
        let json = cmd.to_json_value();
        let restored = ClientCommandPing::from_json(&json);
        assert_eq!(restored.timestamp, 0);
    }
}
