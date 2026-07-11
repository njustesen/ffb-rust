use ffb_model::enums::NetCommandId;
use ffb_model::model::Pushback;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandPushback`.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandPushback {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    pub pushback: Option<Pushback>,
}

impl ClientCommandPushback {
    pub fn new() -> Self { Self::default() }

    pub fn with_pushback(pushback: Pushback) -> Self {
        Self { entropy: None, pushback: Some(pushback) }
    }

    pub fn get_pushback(&self) -> Option<&Pushback> { self.pushback.as_ref() }

    /// Java: `ClientCommandPushback.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        if let Some(pushback) = &self.pushback {
            map.insert("pushback".to_string(), pushback.to_json_value());
        }
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandPushback.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            pushback: json.get("pushback").map(Pushback::from_json),
        }
    }
}

impl NetCommand for ClientCommandPushback {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientPushback
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::types::FieldCoordinate;

    #[test]
    fn pushback_stored() {
        let p = Pushback::new("p1", FieldCoordinate::new(3, 5));
        let cmd = ClientCommandPushback::with_pushback(p);
        assert_eq!(cmd.get_pushback().and_then(|p| p.get_player_id()), Some("p1"));
    }

    #[test]
    fn default_none() {
        let cmd = ClientCommandPushback::new();
        assert!(cmd.pushback.is_none());
    }

    #[test]
    fn coordinate_accessible_via_pushback() {
        let coord = FieldCoordinate::new(7, 2);
        let p = Pushback::new("p2", coord);
        let cmd = ClientCommandPushback::with_pushback(p);
        assert_eq!(cmd.get_pushback().and_then(|p| p.get_coordinate()), Some(coord));
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandPushback::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandPushback::default().clone();
    }

    #[test]
    fn get_id_is_client_pushback() {
        assert_eq!(ClientCommandPushback::new().get_id(), NetCommandId::ClientPushback);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_pushback() {
        let p = Pushback::new("p1", FieldCoordinate::new(2, 9));
        let cmd = ClientCommandPushback::with_pushback(p);
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientPushback");
        assert_eq!(json["pushback"]["playerId"], "p1");
        assert_eq!(json["pushback"]["coordinate"], serde_json::json!([2, 9]));
    }

    #[test]
    fn round_trip_with_data() {
        let p = Pushback::new("p5", FieldCoordinate::new(5, 6));
        let mut cmd = ClientCommandPushback::with_pushback(p);
        cmd.entropy = Some(4);
        let json = cmd.to_json_value();
        let restored = ClientCommandPushback::from_json(&json);
        assert_eq!(restored.entropy, Some(4));
        assert_eq!(restored.get_pushback().and_then(|p| p.get_player_id()), Some("p5"));
        assert_eq!(restored.get_pushback().and_then(|p| p.get_coordinate()), Some(FieldCoordinate::new(5, 6)));
    }

    #[test]
    fn round_trip_default() {
        let cmd = ClientCommandPushback::default();
        let json = cmd.to_json_value();
        let restored = ClientCommandPushback::from_json(&json);
        assert!(restored.pushback.is_none());
        assert!(restored.entropy.is_none());
    }
}
