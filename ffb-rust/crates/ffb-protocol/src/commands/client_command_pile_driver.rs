use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of ClientCommandPileDriver (Java field: playerId).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandPileDriver {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    pub player_id: Option<String>,
}

impl ClientCommandPileDriver {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_player_id(id: impl Into<String>) -> Self {
        Self { entropy: None, player_id: Some(id.into()) }
    }

    pub fn get_player_id(&self) -> Option<&str> {
        self.player_id.as_deref()
    }

    /// Java: `ClientCommandPileDriver.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        if let Some(player_id) = &self.player_id {
            map.insert("playerId".to_string(), serde_json::json!(player_id));
        }
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandPileDriver.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            player_id: json.get("playerId").and_then(|v| v.as_str()).map(String::from),
        }
    }
}

impl NetCommand for ClientCommandPileDriver {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientPileDriver
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_no_player_id() {
        let cmd = ClientCommandPileDriver::new();
        assert!(cmd.get_player_id().is_none());
    }

    #[test]
    fn with_player_id_stores_value() {
        let cmd = ClientCommandPileDriver::with_player_id("p-7");
        assert_eq!(cmd.get_player_id(), Some("p-7"));
    }

    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandPileDriver::new()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandPileDriver::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandPileDriver::default());
        assert!(s.contains("ClientCommandPileDriver"));
    }

    #[test]
    fn get_id_is_client_pile_driver() {
        assert_eq!(ClientCommandPileDriver::new().get_id(), NetCommandId::ClientPileDriver);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_player_id() {
        let cmd = ClientCommandPileDriver::with_player_id("p-9");
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientPileDriver");
        assert_eq!(json["playerId"], "p-9");
    }

    #[test]
    fn round_trip_with_data() {
        let mut cmd = ClientCommandPileDriver::with_player_id("p-3");
        cmd.entropy = Some(2);
        let json = cmd.to_json_value();
        let restored = ClientCommandPileDriver::from_json(&json);
        assert_eq!(restored.entropy, Some(2));
        assert_eq!(restored.get_player_id(), Some("p-3"));
    }

    #[test]
    fn round_trip_default() {
        let cmd = ClientCommandPileDriver::default();
        let json = cmd.to_json_value();
        let restored = ClientCommandPileDriver::from_json(&json);
        assert!(restored.player_id.is_none());
        assert!(restored.entropy.is_none());
    }
}
