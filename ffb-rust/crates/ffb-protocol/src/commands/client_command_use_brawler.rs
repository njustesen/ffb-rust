use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of ClientCommandUseBrawler (Java field: targetId).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandUseBrawler {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    pub target_id: Option<String>,
}

impl ClientCommandUseBrawler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_target_id(id: impl Into<String>) -> Self {
        Self { entropy: None, target_id: Some(id.into()) }
    }

    pub fn get_target_id(&self) -> Option<&str> {
        self.target_id.as_deref()
    }

    /// Java: `ClientCommandUseBrawler.toJsonValue()` — wire key is `playerId` (Java writes
    /// `targetId` under `IJsonOption.PLAYER_ID`).
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("playerId".to_string(), serde_json::json!(self.target_id));
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandUseBrawler.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            target_id: json.get("playerId").and_then(|v| v.as_str()).map(|s| s.to_string()),
        }
    }
}

impl NetCommand for ClientCommandUseBrawler {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientUseBrawler
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_no_target_id() {
        let cmd = ClientCommandUseBrawler::new();
        assert!(cmd.get_target_id().is_none());
    }

    #[test]
    fn with_target_id_stores_value() {
        let cmd = ClientCommandUseBrawler::with_target_id("t-1");
        assert_eq!(cmd.get_target_id(), Some("t-1"));
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandUseBrawler::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandUseBrawler::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandUseBrawler::default());
        assert!(s.contains("ClientCommandUseBrawler"));
    }

    #[test]
    fn get_id_is_client_use_brawler() {
        assert_eq!(ClientCommandUseBrawler::new().get_id(), NetCommandId::ClientUseBrawler);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_player_id_key() {
        let cmd = ClientCommandUseBrawler::with_target_id("t-1");
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientUseBrawler");
        assert_eq!(json["playerId"], "t-1");
    }

    #[test]
    fn round_trip_with_target_id_and_entropy() {
        let mut cmd = ClientCommandUseBrawler::with_target_id("t-2");
        cmd.entropy = Some(4);
        let json = cmd.to_json_value();
        let restored = ClientCommandUseBrawler::from_json(&json);
        assert_eq!(restored.entropy, Some(4));
        assert_eq!(restored.get_target_id(), Some("t-2"));
    }

    #[test]
    fn round_trip_with_no_target_id() {
        let cmd = ClientCommandUseBrawler::new();
        let json = cmd.to_json_value();
        let restored = ClientCommandUseBrawler::from_json(&json);
        assert!(restored.get_target_id().is_none());
    }
}
