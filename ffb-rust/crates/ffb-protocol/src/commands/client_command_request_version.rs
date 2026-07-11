use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of ClientCommandRequestVersion (Java: no fields besides
/// the inherited `ClientCommand.fEntropy`).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandRequestVersion {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
}

impl ClientCommandRequestVersion {
    pub fn new() -> Self {
        Self::default()
    }

    /// Java: `ClientCommandRequestVersion` has no `toJsonValue()` override —
    /// it inherits `ClientCommand.toJsonValue()` unchanged.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        serde_json::Value::Object(base.base_json_fields(self.get_id()))
    }

    /// Java: `ClientCommandRequestVersion.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self { entropy: base.entropy }
    }
}

impl NetCommand for ClientCommandRequestVersion {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientRequestVersion
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_construct() {
        let _cmd = ClientCommandRequestVersion::new();
    }

    #[test]
    fn default_works() {
        let _cmd = ClientCommandRequestVersion::default();
    }

    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandRequestVersion::new()).is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandRequestVersion::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandRequestVersion::default());
        assert!(s.contains("ClientCommandRequestVersion"));
    }

    #[test]
    fn get_id_is_client_request_version() {
        assert_eq!(ClientCommandRequestVersion::new().get_id(), NetCommandId::ClientRequestVersion);
    }

    #[test]
    fn to_json_value_has_net_command_id() {
        let cmd = ClientCommandRequestVersion::new();
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientRequestVersion");
    }

    #[test]
    fn round_trip_with_entropy() {
        let mut cmd = ClientCommandRequestVersion::new();
        cmd.entropy = Some(5);
        let json = cmd.to_json_value();
        let restored = ClientCommandRequestVersion::from_json(&json);
        assert_eq!(restored.entropy, Some(5));
    }

    #[test]
    fn round_trip_default() {
        let cmd = ClientCommandRequestVersion::default();
        let json = cmd.to_json_value();
        let restored = ClientCommandRequestVersion::from_json(&json);
        assert!(restored.entropy.is_none());
    }
}
