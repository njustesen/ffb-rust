use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandUseHatred`.
/// Sent when a player uses Hatred to make an extra block.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandUseHatred {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `targetId`
    pub target_id: Option<String>,
}

impl ClientCommandUseHatred {
    pub fn new() -> Self { Self::default() }
    pub fn with_target(target_id: impl Into<String>) -> Self {
        Self { entropy: None, target_id: Some(target_id.into()) }
    }
    pub fn get_target_id(&self) -> Option<&str> { self.target_id.as_deref() }

    /// Java: `ClientCommandUseHatred.toJsonValue()` (calls `super.toJsonValue()` first).
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        if let Some(target_id) = &self.target_id {
            map.insert("playerId".to_string(), serde_json::json!(target_id));
        }
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandUseHatred.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            target_id: json.get("playerId").and_then(|v| v.as_str()).map(|s| s.to_string()),
        }
    }
}

impl NetCommand for ClientCommandUseHatred {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientUseHatred
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn target_stored() {
        let cmd = ClientCommandUseHatred::with_target("p1");
        assert_eq!(cmd.get_target_id(), Some("p1"));
    }
    #[test]
    fn default_none() {
        assert!(ClientCommandUseHatred::new().target_id.is_none());
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandUseHatred::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandUseHatred::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandUseHatred::default());
        assert!(s.contains("ClientCommandUseHatred"));
    }

    #[test]
    fn get_id_is_client_use_hatred() {
        assert_eq!(ClientCommandUseHatred::new().get_id(), NetCommandId::ClientUseHatred);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_player_id() {
        let cmd = ClientCommandUseHatred::with_target("p9");
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientUseHatred");
        assert_eq!(json["playerId"], "p9");
    }

    #[test]
    fn round_trip_with_target_and_entropy() {
        let mut cmd = ClientCommandUseHatred::with_target("p2");
        cmd.entropy = Some(4);
        let json = cmd.to_json_value();
        let restored = ClientCommandUseHatred::from_json(&json);
        assert_eq!(restored.entropy, Some(4));
        assert_eq!(restored.target_id.as_deref(), Some("p2"));
    }

    #[test]
    fn round_trip_with_no_target() {
        let cmd = ClientCommandUseHatred::new();
        let json = cmd.to_json_value();
        let restored = ClientCommandUseHatred::from_json(&json);
        assert!(restored.target_id.is_none());
    }
}
