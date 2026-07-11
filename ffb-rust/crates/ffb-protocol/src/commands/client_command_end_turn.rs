use std::collections::HashMap;
use ffb_model::enums::{NetCommandId, TurnMode};
use ffb_model::types::FieldCoordinate;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandEndTurn`.
/// Sent when the active coach ends their team's turn.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandEndTurn {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `turnMode`
    pub turn_mode: Option<TurnMode>,
    /// Java: `playerCoordinates` — snapshot of player positions for client-side sync.
    pub player_coordinates: HashMap<String, FieldCoordinate>,
}

impl ClientCommandEndTurn {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_turn_mode(&self) -> Option<TurnMode> { self.turn_mode }
    pub fn get_player_coordinates(&self) -> &HashMap<String, FieldCoordinate> { &self.player_coordinates }

    /// Java: `ClientCommandEndTurn.toJsonValue()` (calls `super.toJsonValue()` first).
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        if let Some(turn_mode) = self.turn_mode {
            map.insert("turnMode".to_string(), serde_json::json!(turn_mode.name()));
        }
        let coords: serde_json::Map<String, serde_json::Value> = self
            .player_coordinates
            .iter()
            .map(|(id, fc)| (id.clone(), fc.to_json_value()))
            .collect();
        map.insert("playersAtCoordinates".to_string(), serde_json::Value::Object(coords));
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandEndTurn.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        let player_coordinates = json
            .get("playersAtCoordinates")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(id, v)| FieldCoordinate::from_json(v).map(|fc| (id.clone(), fc)))
                    .collect()
            })
            .unwrap_or_default();
        Self {
            entropy: base.entropy,
            turn_mode: json.get("turnMode").and_then(|v| v.as_str()).and_then(TurnMode::from_name),
            player_coordinates,
        }
    }
}

impl NetCommand for ClientCommandEndTurn {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientEndTurn
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_empty() {
        let cmd = ClientCommandEndTurn::new();
        assert!(cmd.turn_mode.is_none());
        assert!(cmd.player_coordinates.is_empty());
    }

    #[test]
    fn turn_mode_stored() {
        let mut cmd = ClientCommandEndTurn::new();
        cmd.turn_mode = Some(TurnMode::Regular);
        assert_eq!(cmd.get_turn_mode(), Some(TurnMode::Regular));
    }

    #[test]
    fn player_coordinates_stored() {
        let mut cmd = ClientCommandEndTurn::new();
        cmd.player_coordinates.insert("p1".into(), FieldCoordinate::new(5, 5));
        assert_eq!(cmd.player_coordinates.len(), 1);
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandEndTurn::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandEndTurn::default().clone();
    }

    #[test]
    fn get_id_is_client_end_turn() {
        assert_eq!(ClientCommandEndTurn::new().get_id(), NetCommandId::ClientEndTurn);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_turn_mode() {
        let mut cmd = ClientCommandEndTurn::new();
        cmd.turn_mode = Some(TurnMode::Blitz);
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientEndTurn");
        assert_eq!(json["turnMode"], "blitz");
    }

    #[test]
    fn round_trip_with_coordinates_and_entropy() {
        let mut cmd = ClientCommandEndTurn::new();
        cmd.entropy = Some(9);
        cmd.turn_mode = Some(TurnMode::Regular);
        cmd.player_coordinates.insert("p1".into(), FieldCoordinate::new(3, 4));
        let json = cmd.to_json_value();
        let restored = ClientCommandEndTurn::from_json(&json);
        assert_eq!(restored.entropy, Some(9));
        assert_eq!(restored.turn_mode, Some(TurnMode::Regular));
        assert_eq!(restored.player_coordinates.get("p1"), Some(&FieldCoordinate::new(3, 4)));
    }

    #[test]
    fn round_trip_with_no_turn_mode_and_empty_coordinates() {
        let cmd = ClientCommandEndTurn::new();
        let json = cmd.to_json_value();
        let restored = ClientCommandEndTurn::from_json(&json);
        assert!(restored.turn_mode.is_none());
        assert!(restored.player_coordinates.is_empty());
    }
}
