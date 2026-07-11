use ffb_model::enums::NetCommandId;
use ffb_model::types::FieldCoordinate;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandPass`.
/// Sent when a player attempts a pass.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandPass {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `fActingPlayerId`
    pub acting_player_id: Option<String>,
    /// Java: `fTargetCoordinate`
    pub target_coordinate: Option<FieldCoordinate>,
}

impl ClientCommandPass {
    pub fn new(acting_player_id: impl Into<String>, target_coordinate: FieldCoordinate) -> Self {
        Self {
            entropy: None,
            acting_player_id: Some(acting_player_id.into()),
            target_coordinate: Some(target_coordinate),
        }
    }

    pub fn get_acting_player_id(&self) -> Option<&str> { self.acting_player_id.as_deref() }
    pub fn get_target_coordinate(&self) -> Option<FieldCoordinate> { self.target_coordinate }

    /// Java: `ClientCommandPass.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        if let Some(acting_player_id) = &self.acting_player_id {
            map.insert("actingPlayerId".to_string(), serde_json::json!(acting_player_id));
        }
        if let Some(target_coordinate) = self.target_coordinate {
            map.insert("targetCoordinate".to_string(), target_coordinate.to_json_value());
        }
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandPass.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            acting_player_id: json.get("actingPlayerId").and_then(|v| v.as_str()).map(String::from),
            target_coordinate: json.get("targetCoordinate").and_then(FieldCoordinate::from_json),
        }
    }
}

impl NetCommand for ClientCommandPass {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientPass
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let coord = FieldCoordinate::new(10, 5);
        let cmd = ClientCommandPass::new("thrower1", coord);
        assert_eq!(cmd.get_acting_player_id(), Some("thrower1"));
        assert_eq!(cmd.get_target_coordinate(), Some(coord));
    }

    #[test]
    fn default_is_empty() {
        let cmd = ClientCommandPass::default();
        assert!(cmd.acting_player_id.is_none());
        assert!(cmd.target_coordinate.is_none());
    }

    #[test]
    fn new_with_coord() {
        let coord = FieldCoordinate::new(0, 0);
        let cmd = ClientCommandPass::new("p1", coord);
        assert!(cmd.get_acting_player_id().is_some());
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandPass::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandPass::default().clone();
    }

    #[test]
    fn get_id_is_client_pass() {
        assert_eq!(ClientCommandPass::default().get_id(), NetCommandId::ClientPass);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_target_coordinate() {
        let cmd = ClientCommandPass::new("p1", FieldCoordinate::new(9, 1));
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientPass");
        assert_eq!(json["targetCoordinate"], serde_json::json!([9, 1]));
    }

    #[test]
    fn round_trip_with_data() {
        let mut cmd = ClientCommandPass::new("p2", FieldCoordinate::new(4, 4));
        cmd.entropy = Some(9);
        let json = cmd.to_json_value();
        let restored = ClientCommandPass::from_json(&json);
        assert_eq!(restored.entropy, Some(9));
        assert_eq!(restored.get_acting_player_id(), Some("p2"));
        assert_eq!(restored.get_target_coordinate(), Some(FieldCoordinate::new(4, 4)));
    }

    #[test]
    fn round_trip_default() {
        let cmd = ClientCommandPass::default();
        let json = cmd.to_json_value();
        let restored = ClientCommandPass::from_json(&json);
        assert!(restored.acting_player_id.is_none());
        assert!(restored.target_coordinate.is_none());
        assert!(restored.entropy.is_none());
    }
}
