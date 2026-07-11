use ffb_model::enums::NetCommandId;
use ffb_model::types::FieldCoordinate;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandSwoop`.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandSwoop {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `fTargetCoordinate`
    pub target_coordinate: Option<FieldCoordinate>,
    /// Java: `fActingPlayerId`
    pub acting_player_id: Option<String>,
}

impl ClientCommandSwoop {
    pub fn new() -> Self { Self::default() }

    pub fn with_players_and_target(
        acting_player_id: impl Into<String>,
        target_coordinate: FieldCoordinate,
    ) -> Self {
        Self {
            entropy: None,
            acting_player_id: Some(acting_player_id.into()),
            target_coordinate: Some(target_coordinate),
        }
    }

    pub fn get_target_coordinate(&self) -> Option<FieldCoordinate> { self.target_coordinate }
    pub fn get_acting_player_id(&self) -> Option<&str> { self.acting_player_id.as_deref() }

    /// Java: `ClientCommandSwoop.toJsonValue()`.
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

    /// Java: `ClientCommandSwoop.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            acting_player_id: json.get("actingPlayerId").and_then(|v| v.as_str()).map(String::from),
            target_coordinate: json.get("targetCoordinate").and_then(FieldCoordinate::from_json),
        }
    }
}

impl NetCommand for ClientCommandSwoop {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientSwoop
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let coord = FieldCoordinate::new(7, 3);
        let cmd = ClientCommandSwoop::with_players_and_target("attacker1", coord);
        assert_eq!(cmd.get_acting_player_id(), Some("attacker1"));
        assert_eq!(cmd.get_target_coordinate(), Some(coord));
    }

    #[test]
    fn default_is_empty() {
        let cmd = ClientCommandSwoop::new();
        assert!(cmd.acting_player_id.is_none());
        assert!(cmd.target_coordinate.is_none());
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandSwoop::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandSwoop::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandSwoop::default());
        assert!(s.contains("ClientCommandSwoop"));
    }

    #[test]
    fn get_id_is_client_swoop() {
        assert_eq!(ClientCommandSwoop::new().get_id(), NetCommandId::ClientSwoop);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_acting_player_id() {
        let cmd = ClientCommandSwoop::with_players_and_target("p1", FieldCoordinate::new(2, 5));
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientSwoop");
        assert_eq!(json["actingPlayerId"], "p1");
    }

    #[test]
    fn round_trip_with_all_fields() {
        let mut cmd = ClientCommandSwoop::with_players_and_target("p1", FieldCoordinate::new(2, 5));
        cmd.entropy = Some(11);
        let json = cmd.to_json_value();
        let restored = ClientCommandSwoop::from_json(&json);
        assert_eq!(restored.entropy, Some(11));
        assert_eq!(restored.acting_player_id.as_deref(), Some("p1"));
        assert_eq!(restored.target_coordinate, Some(FieldCoordinate::new(2, 5)));
    }

    #[test]
    fn round_trip_with_no_fields() {
        let cmd = ClientCommandSwoop::new();
        let json = cmd.to_json_value();
        let restored = ClientCommandSwoop::from_json(&json);
        assert!(restored.acting_player_id.is_none());
        assert!(restored.target_coordinate.is_none());
    }
}
