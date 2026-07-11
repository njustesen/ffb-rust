use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of ClientCommandTargetSelected (Java field: targetPlayerId).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandTargetSelected {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    pub target_player_id: Option<String>,
}

impl ClientCommandTargetSelected {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_target(id: impl Into<String>) -> Self {
        Self { entropy: None, target_player_id: Some(id.into()) }
    }

    pub fn get_target_player_id(&self) -> Option<&str> {
        self.target_player_id.as_deref()
    }

    /// Java: `ClientCommandTargetSelected.toJsonValue()`. Java writes the
    /// field under `IJsonOption.PLAYER_ID` (wire key `"playerId"`), not
    /// `TARGET_PLAYER_ID` (`"targetPlayerId"`) despite the Java field name.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        if let Some(target_player_id) = &self.target_player_id {
            map.insert("playerId".to_string(), serde_json::json!(target_player_id));
        }
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandTargetSelected.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            target_player_id: json.get("playerId").and_then(|v| v.as_str()).map(String::from),
        }
    }
}

impl NetCommand for ClientCommandTargetSelected {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientTargetSelected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_no_target() {
        let cmd = ClientCommandTargetSelected::new();
        assert!(cmd.get_target_player_id().is_none());
    }

    #[test]
    fn with_target_stores_value() {
        let cmd = ClientCommandTargetSelected::with_target("player-123");
        assert_eq!(cmd.get_target_player_id(), Some("player-123"));
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandTargetSelected::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandTargetSelected::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandTargetSelected::default());
        assert!(s.contains("ClientCommandTargetSelected"));
    }

    #[test]
    fn get_id_is_client_target_selected() {
        assert_eq!(ClientCommandTargetSelected::new().get_id(), NetCommandId::ClientTargetSelected);
    }

    #[test]
    fn to_json_value_uses_player_id_wire_key() {
        let cmd = ClientCommandTargetSelected::with_target("player-123");
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "targetSelected");
        assert_eq!(json["playerId"], "player-123");
    }

    #[test]
    fn round_trip_with_target_and_entropy() {
        let mut cmd = ClientCommandTargetSelected::with_target("player-123");
        cmd.entropy = Some(2);
        let json = cmd.to_json_value();
        let restored = ClientCommandTargetSelected::from_json(&json);
        assert_eq!(restored.entropy, Some(2));
        assert_eq!(restored.target_player_id.as_deref(), Some("player-123"));
    }

    #[test]
    fn round_trip_with_no_target() {
        let cmd = ClientCommandTargetSelected::new();
        let json = cmd.to_json_value();
        let restored = ClientCommandTargetSelected::from_json(&json);
        assert!(restored.target_player_id.is_none());
    }
}
