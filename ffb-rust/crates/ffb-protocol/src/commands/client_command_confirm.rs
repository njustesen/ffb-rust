use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;
use ffb_model::enums::NetCommandId;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandConfirm`.
/// Sent when a coach confirms a dialog choice (no payload).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandConfirm {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
}

impl ClientCommandConfirm {
    pub fn new() -> Self { Self { entropy: None } }

    /// Java: `ClientCommandConfirm.toJsonValue()` (only calls `super.toJsonValue()`).
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let map = base.base_json_fields(self.get_id());
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandConfirm.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self { entropy: base.entropy }
    }
}

impl NetCommand for ClientCommandConfirm {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientConfirm
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_construct() {
        let _ = ClientCommandConfirm::new();
    }

    #[test]
    fn default_same_as_new() {
        let _ = ClientCommandConfirm::default();
    }

    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandConfirm::new()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandConfirm::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandConfirm::default());
        assert!(s.contains("ClientCommandConfirm"));
    }

    #[test]
    fn get_id_is_client_confirm() {
        assert_eq!(ClientCommandConfirm::new().get_id(), NetCommandId::ClientConfirm);
    }

    #[test]
    fn to_json_value_has_net_command_id() {
        let json = ClientCommandConfirm::new().to_json_value();
        assert_eq!(json["netCommandId"], "clientConfirm");
    }

    #[test]
    fn round_trip_with_entropy() {
        let mut cmd = ClientCommandConfirm::new();
        cmd.entropy = Some(11);
        let json = cmd.to_json_value();
        let restored = ClientCommandConfirm::from_json(&json);
        assert_eq!(restored.entropy, Some(11));
    }

    #[test]
    fn round_trip_with_no_entropy() {
        let cmd = ClientCommandConfirm::new();
        let json = cmd.to_json_value();
        let restored = ClientCommandConfirm::from_json(&json);
        assert!(restored.entropy.is_none());
    }
}
