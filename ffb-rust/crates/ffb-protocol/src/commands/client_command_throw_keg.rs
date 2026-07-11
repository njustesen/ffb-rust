use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of ClientCommandThrowKeg (Java field: playerId).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandThrowKeg {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    pub player_id: Option<String>,
}

impl ClientCommandThrowKeg {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_player_id(id: impl Into<String>) -> Self {
        Self { entropy: None, player_id: Some(id.into()) }
    }

    pub fn get_player_id(&self) -> Option<&str> {
        self.player_id.as_deref()
    }

    /// Java: `ClientCommandThrowKeg.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("playerId".to_string(), serde_json::json!(self.player_id));
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandThrowKeg.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            player_id: json.get("playerId").and_then(|v| v.as_str()).map(|s| s.to_string()),
        }
    }
}

impl NetCommand for ClientCommandThrowKeg {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientThrowKeg
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_no_player_id() {
        let cmd = ClientCommandThrowKeg::new();
        assert!(cmd.get_player_id().is_none());
    }

    #[test]
    fn with_player_id_stores_value() {
        let cmd = ClientCommandThrowKeg::with_player_id("p-42");
        assert_eq!(cmd.get_player_id(), Some("p-42"));
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandThrowKeg::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandThrowKeg::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandThrowKeg::default());
        assert!(s.contains("ClientCommandThrowKeg"));
    }

    #[test]
    fn get_id_is_client_throw_keg() {
        assert_eq!(ClientCommandThrowKeg::new().get_id(), NetCommandId::ClientThrowKeg);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_player_id() {
        let cmd = ClientCommandThrowKeg::with_player_id("p-1");
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientThrowKeg");
        assert_eq!(json["playerId"], "p-1");
    }

    #[test]
    fn round_trip_with_player_id_and_entropy() {
        let mut cmd = ClientCommandThrowKeg::with_player_id("p-2");
        cmd.entropy = Some(5);
        let json = cmd.to_json_value();
        let restored = ClientCommandThrowKeg::from_json(&json);
        assert_eq!(restored.entropy, Some(5));
        assert_eq!(restored.get_player_id(), Some("p-2"));
    }

    #[test]
    fn round_trip_with_no_player_id() {
        let cmd = ClientCommandThrowKeg::new();
        let json = cmd.to_json_value();
        let restored = ClientCommandThrowKeg::from_json(&json);
        assert!(restored.get_player_id().is_none());
    }
}
