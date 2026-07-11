use ffb_model::enums::NetCommandId;
use ffb_model::types::FieldCoordinate;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandBlitzMove`.
/// Sent when a player performs a blitz move.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandBlitzMove {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `fActingPlayerId`
    pub acting_player_id: Option<String>,
    /// Java: `fCoordinateFrom`
    pub coordinate_from: Option<FieldCoordinate>,
    /// Java: `fCoordinatesTo`
    pub coordinates_to: Vec<FieldCoordinate>,
}

impl ClientCommandBlitzMove {
    pub fn new() -> Self { Self::default() }

    pub fn with_move(
        acting_player_id: impl Into<String>,
        coordinate_from: FieldCoordinate,
        coordinates_to: Vec<FieldCoordinate>,
    ) -> Self {
        Self {
            entropy: None,
            acting_player_id: Some(acting_player_id.into()),
            coordinate_from: Some(coordinate_from),
            coordinates_to,
        }
    }

    pub fn get_acting_player_id(&self) -> Option<&str> { self.acting_player_id.as_deref() }
    pub fn get_coordinate_from(&self) -> Option<FieldCoordinate> { self.coordinate_from }
    pub fn get_coordinates_to(&self) -> &[FieldCoordinate] { &self.coordinates_to }

    /// Java: `ClientCommandBlitzMove.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        if let Some(acting_player_id) = &self.acting_player_id {
            map.insert("actingPlayerId".to_string(), serde_json::json!(acting_player_id));
        }
        if let Some(coordinate_from) = self.coordinate_from {
            map.insert("coordinateFrom".to_string(), coordinate_from.to_json_value());
        }
        let coordinates_to: Vec<serde_json::Value> =
            self.coordinates_to.iter().map(|fc| fc.to_json_value()).collect();
        map.insert("coordinatesTo".to_string(), serde_json::Value::Array(coordinates_to));
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandBlitzMove.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        let coordinates_to = json
            .get("coordinatesTo")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(FieldCoordinate::from_json).collect())
            .unwrap_or_default();
        Self {
            entropy: base.entropy,
            acting_player_id: json.get("actingPlayerId").and_then(|v| v.as_str()).map(String::from),
            coordinate_from: json.get("coordinateFrom").and_then(FieldCoordinate::from_json),
            coordinates_to,
        }
    }
}

impl NetCommand for ClientCommandBlitzMove {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientBlitzMove
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored_correctly() {
        let from = FieldCoordinate::new(1, 1);
        let to = vec![FieldCoordinate::new(2, 2), FieldCoordinate::new(3, 3)];
        let cmd = ClientCommandBlitzMove::with_move("p1", from, to.clone());
        assert_eq!(cmd.get_acting_player_id(), Some("p1"));
        assert_eq!(cmd.get_coordinate_from(), Some(from));
        assert_eq!(cmd.get_coordinates_to().len(), 2);
    }

    #[test]
    fn default_all_none() {
        let cmd = ClientCommandBlitzMove::new();
        assert!(cmd.acting_player_id.is_none());
        assert!(cmd.coordinate_from.is_none());
        assert!(cmd.coordinates_to.is_empty());
    }

    #[test]
    fn coordinates_to_slice_matches_input() {
        let from = FieldCoordinate::new(0, 0);
        let to = vec![FieldCoordinate::new(1, 0)];
        let cmd = ClientCommandBlitzMove::with_move("p2", from, to.clone());
        assert_eq!(cmd.get_coordinates_to(), to.as_slice());
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandBlitzMove::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandBlitzMove::default().clone();
    }

    #[test]
    fn get_id_is_client_blitz_move() {
        assert_eq!(ClientCommandBlitzMove::new().get_id(), NetCommandId::ClientBlitzMove);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_acting_player_id() {
        let cmd = ClientCommandBlitzMove::with_move("p1", FieldCoordinate::new(1, 1), vec![]);
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientBlitzMove");
        assert_eq!(json["actingPlayerId"], "p1");
    }

    #[test]
    fn round_trip_with_populated_data() {
        let mut cmd = ClientCommandBlitzMove::with_move(
            "p1",
            FieldCoordinate::new(1, 1),
            vec![FieldCoordinate::new(2, 2), FieldCoordinate::new(3, 3)],
        );
        cmd.entropy = Some(7);
        let json = cmd.to_json_value();
        let restored = ClientCommandBlitzMove::from_json(&json);
        assert_eq!(restored.entropy, Some(7));
        assert_eq!(restored.acting_player_id.as_deref(), Some("p1"));
        assert_eq!(restored.coordinate_from, Some(FieldCoordinate::new(1, 1)));
        assert_eq!(restored.coordinates_to, vec![FieldCoordinate::new(2, 2), FieldCoordinate::new(3, 3)]);
    }

    #[test]
    fn round_trip_with_default_data() {
        let cmd = ClientCommandBlitzMove::new();
        let json = cmd.to_json_value();
        let restored = ClientCommandBlitzMove::from_json(&json);
        assert!(restored.entropy.is_none());
        assert!(restored.acting_player_id.is_none());
        assert!(restored.coordinate_from.is_none());
        assert!(restored.coordinates_to.is_empty());
    }
}
