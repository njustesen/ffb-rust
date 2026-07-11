use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;
use ffb_model::enums::NetCommandId;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandCloseSession`.
/// Sent when a client disconnects or closes their session (no payload).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandCloseSession {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
}

impl ClientCommandCloseSession {
    pub fn new() -> Self { Self { entropy: None } }

    /// Java: `ClientCommandCloseSession.toJsonValue()` (only calls `super.toJsonValue()`).
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let map = base.base_json_fields(self.get_id());
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandCloseSession.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self { entropy: base.entropy }
    }
}

impl NetCommand for ClientCommandCloseSession {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientCloseSession
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn can_construct() { let _ = ClientCommandCloseSession::new(); }

    #[test]
    fn default_same_as_new() { let _ = ClientCommandCloseSession::default(); }

    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandCloseSession::new()).is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandCloseSession::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandCloseSession::default());
        assert!(s.contains("ClientCommandCloseSession"));
    }

    #[test]
    fn get_id_is_client_close_session() {
        assert_eq!(ClientCommandCloseSession::new().get_id(), NetCommandId::ClientCloseSession);
    }

    #[test]
    fn to_json_value_has_net_command_id() {
        let json = ClientCommandCloseSession::new().to_json_value();
        assert_eq!(json["netCommandId"], "clientCloseSession");
    }

    #[test]
    fn round_trip_with_entropy() {
        let mut cmd = ClientCommandCloseSession::new();
        cmd.entropy = Some(5);
        let json = cmd.to_json_value();
        let restored = ClientCommandCloseSession::from_json(&json);
        assert_eq!(restored.entropy, Some(5));
    }

    #[test]
    fn round_trip_with_no_entropy() {
        let cmd = ClientCommandCloseSession::new();
        let json = cmd.to_json_value();
        let restored = ClientCommandCloseSession::from_json(&json);
        assert!(restored.entropy.is_none());
    }
}
