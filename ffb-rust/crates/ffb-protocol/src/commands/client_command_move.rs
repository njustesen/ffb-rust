use ffb_model::enums::NetCommandId;
use ffb_model::types::FieldCoordinate;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandMove`.
/// Sent when a player moves through a sequence of squares.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandMove {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `fActingPlayerId`
    pub acting_player_id: Option<String>,
    /// Java: `fCoordinateFrom`
    pub coordinate_from: Option<FieldCoordinate>,
    /// Java: `fCoordinatesTo`
    pub coordinates_to: Vec<FieldCoordinate>,
    /// Java: `ballAndChainRrSetting`
    pub ball_and_chain_rr_setting: Option<String>,
}

impl ClientCommandMove {
    pub fn new(
        acting_player_id: impl Into<String>,
        coordinate_from: FieldCoordinate,
        coordinates_to: Vec<FieldCoordinate>,
        ball_and_chain_rr_setting: Option<String>,
    ) -> Self {
        Self {
            entropy: None,
            acting_player_id: Some(acting_player_id.into()),
            coordinate_from: Some(coordinate_from),
            coordinates_to,
            ball_and_chain_rr_setting,
        }
    }

    pub fn get_acting_player_id(&self) -> Option<&str> { self.acting_player_id.as_deref() }
    pub fn get_coordinate_from(&self) -> Option<FieldCoordinate> { self.coordinate_from }
    pub fn get_coordinates_to(&self) -> &[FieldCoordinate] { &self.coordinates_to }
    pub fn get_ball_and_chain_rr_setting(&self) -> Option<&str> { self.ball_and_chain_rr_setting.as_deref() }

    /// Java: `ClientCommandMove.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        if let Some(acting_player_id) = &self.acting_player_id {
            map.insert("actingPlayerId".to_string(), serde_json::json!(acting_player_id));
        }
        if let Some(coordinate_from) = self.coordinate_from {
            map.insert("coordinateFrom".to_string(), coordinate_from.to_json_value());
        }
        map.insert(
            "coordinatesTo".to_string(),
            serde_json::Value::Array(self.coordinates_to.iter().map(|c| c.to_json_value()).collect()),
        );
        if let Some(setting) = &self.ball_and_chain_rr_setting {
            map.insert("ballAndChainReRollSetting".to_string(), serde_json::json!(setting));
        }
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandMove.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            acting_player_id: json.get("actingPlayerId").and_then(|v| v.as_str()).map(String::from),
            coordinate_from: json.get("coordinateFrom").and_then(FieldCoordinate::from_json),
            coordinates_to: json
                .get("coordinatesTo")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(FieldCoordinate::from_json).collect())
                .unwrap_or_default(),
            ball_and_chain_rr_setting: json.get("ballAndChainReRollSetting").and_then(|v| v.as_str()).map(String::from),
        }
    }
}

impl NetCommand for ClientCommandMove {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientMove
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let from = FieldCoordinate::new(3, 4);
        let to = vec![FieldCoordinate::new(4, 4), FieldCoordinate::new(5, 4)];
        let cmd = ClientCommandMove::new("p1", from, to.clone(), None);
        assert_eq!(cmd.get_acting_player_id(), Some("p1"));
        assert_eq!(cmd.get_coordinate_from(), Some(from));
        assert_eq!(cmd.get_coordinates_to().len(), 2);
        assert!(cmd.get_ball_and_chain_rr_setting().is_none());
    }

    #[test]
    fn ball_and_chain_setting() {
        let cmd = ClientCommandMove::new("p1", FieldCoordinate::new(1, 1), vec![], Some("ALWAYS".into()));
        assert_eq!(cmd.get_ball_and_chain_rr_setting(), Some("ALWAYS"));
    }

    #[test]
    fn default_empty() {
        let cmd = ClientCommandMove::default();
        assert!(cmd.acting_player_id.is_none());
        assert!(cmd.coordinates_to.is_empty());
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandMove::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandMove::default().clone();
    }

    #[test]
    fn get_id_is_client_move() {
        assert_eq!(ClientCommandMove::default().get_id(), NetCommandId::ClientMove);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_coordinates() {
        let from = FieldCoordinate::new(2, 2);
        let to = vec![FieldCoordinate::new(3, 2)];
        let cmd = ClientCommandMove::new("p1", from, to, Some("ALWAYS".into()));
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientMove");
        assert_eq!(json["coordinateFrom"], serde_json::json!([2, 2]));
        assert_eq!(json["coordinatesTo"], serde_json::json!([[3, 2]]));
        assert_eq!(json["ballAndChainReRollSetting"], "ALWAYS");
    }

    #[test]
    fn round_trip_with_data() {
        let from = FieldCoordinate::new(6, 7);
        let to = vec![FieldCoordinate::new(7, 7), FieldCoordinate::new(8, 7)];
        let mut cmd = ClientCommandMove::new("p9", from, to.clone(), Some("NEVER".into()));
        cmd.entropy = Some(4);
        let json = cmd.to_json_value();
        let restored = ClientCommandMove::from_json(&json);
        assert_eq!(restored.entropy, Some(4));
        assert_eq!(restored.get_acting_player_id(), Some("p9"));
        assert_eq!(restored.get_coordinate_from(), Some(from));
        assert_eq!(restored.get_coordinates_to(), to.as_slice());
        assert_eq!(restored.get_ball_and_chain_rr_setting(), Some("NEVER"));
    }

    #[test]
    fn round_trip_default() {
        let cmd = ClientCommandMove::default();
        let json = cmd.to_json_value();
        let restored = ClientCommandMove::from_json(&json);
        assert!(restored.acting_player_id.is_none());
        assert!(restored.coordinate_from.is_none());
        assert!(restored.coordinates_to.is_empty());
        assert!(restored.ball_and_chain_rr_setting.is_none());
        assert!(restored.entropy.is_none());
    }
}
