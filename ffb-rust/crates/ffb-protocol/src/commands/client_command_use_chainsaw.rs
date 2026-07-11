use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of ClientCommandUseChainsaw (Java field: usingChainsaw).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandUseChainsaw {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    pub using_chainsaw: bool,
}

impl ClientCommandUseChainsaw {
    pub fn new(using_chainsaw: bool) -> Self {
        Self { entropy: None, using_chainsaw }
    }

    pub fn is_using_chainsaw(&self) -> bool {
        self.using_chainsaw
    }

    /// Java: `ClientCommandUseChainsaw.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("usingChainsaw".to_string(), serde_json::json!(self.using_chainsaw));
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandUseChainsaw.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            using_chainsaw: json.get("usingChainsaw").and_then(|v| v.as_bool()).unwrap_or(false),
        }
    }
}

impl NetCommand for ClientCommandUseChainsaw {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientUseChainsaw
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_true_stores_true() {
        let cmd = ClientCommandUseChainsaw::new(true);
        assert!(cmd.is_using_chainsaw());
    }

    #[test]
    fn new_false_stores_false() {
        let cmd = ClientCommandUseChainsaw::new(false);
        assert!(!cmd.is_using_chainsaw());
    }

    #[test]
    fn default_is_false() {
        let cmd = ClientCommandUseChainsaw::default();
        assert!(!cmd.is_using_chainsaw());
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandUseChainsaw::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandUseChainsaw::default().clone();
    }

    #[test]
    fn get_id_is_client_use_chainsaw() {
        assert_eq!(ClientCommandUseChainsaw::default().get_id(), NetCommandId::ClientUseChainsaw);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_using_chainsaw() {
        let cmd = ClientCommandUseChainsaw::new(true);
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientUseChainsaw");
        assert_eq!(json["usingChainsaw"], true);
    }

    #[test]
    fn round_trip_with_true_and_entropy() {
        let mut cmd = ClientCommandUseChainsaw::new(true);
        cmd.entropy = Some(1);
        let json = cmd.to_json_value();
        let restored = ClientCommandUseChainsaw::from_json(&json);
        assert_eq!(restored.entropy, Some(1));
        assert!(restored.is_using_chainsaw());
    }

    #[test]
    fn round_trip_with_default_false() {
        let cmd = ClientCommandUseChainsaw::default();
        let json = cmd.to_json_value();
        let restored = ClientCommandUseChainsaw::from_json(&json);
        assert!(!restored.is_using_chainsaw());
    }
}
