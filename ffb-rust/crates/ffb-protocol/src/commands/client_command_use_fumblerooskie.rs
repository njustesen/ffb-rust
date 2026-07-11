use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of ClientCommandUseFumblerooskie (Java: no fields).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandUseFumblerooskie {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
}

impl ClientCommandUseFumblerooskie {
    pub fn new() -> Self {
        Self::default()
    }

    /// Java: `ClientCommandUseFumblerooskie.toJsonValue()` (inherited from `ClientCommand`, no override).
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let map = base.base_json_fields(self.get_id());
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandUseFumblerooskie.initFrom(source, jsonValue)` (inherited from `ClientCommand`).
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self { entropy: base.entropy }
    }
}

impl NetCommand for ClientCommandUseFumblerooskie {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientUseFumblerooskie
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_construct() {
        let _cmd = ClientCommandUseFumblerooskie::new();
    }

    #[test]
    fn default_works() {
        let _cmd = ClientCommandUseFumblerooskie::default();
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandUseFumblerooskie::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandUseFumblerooskie::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandUseFumblerooskie::default());
        assert!(s.contains("ClientCommandUseFumblerooskie"));
    }

    #[test]
    fn get_id_is_client_use_fumblerooskie() {
        assert_eq!(ClientCommandUseFumblerooskie::new().get_id(), NetCommandId::ClientUseFumblerooskie);
    }

    #[test]
    fn to_json_value_has_net_command_id() {
        let json = ClientCommandUseFumblerooskie::new().to_json_value();
        assert_eq!(json["netCommandId"], "clientUseFumblerooskie");
    }

    #[test]
    fn round_trip_with_entropy() {
        let mut cmd = ClientCommandUseFumblerooskie::new();
        cmd.entropy = Some(5);
        let json = cmd.to_json_value();
        let restored = ClientCommandUseFumblerooskie::from_json(&json);
        assert_eq!(restored.entropy, Some(5));
    }

    #[test]
    fn round_trip_with_no_entropy() {
        let cmd = ClientCommandUseFumblerooskie::new();
        let json = cmd.to_json_value();
        let restored = ClientCommandUseFumblerooskie::from_json(&json);
        assert!(restored.entropy.is_none());
    }
}
