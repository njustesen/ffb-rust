use ffb_model::enums::NetCommandId;
use ffb_model::types::FieldCoordinate;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandSetupPlayer`.
/// Sent when a player is placed during the setup phase.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandSetupPlayer {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `fPlayerId`
    pub player_id: Option<String>,
    /// Java: `fCoordinate`
    pub coordinate: Option<FieldCoordinate>,
}

impl ClientCommandSetupPlayer {
    pub fn new() -> Self { Self::default() }

    pub fn with_placement(
        player_id: impl Into<String>,
        coordinate: FieldCoordinate,
    ) -> Self {
        Self {
            entropy: None,
            player_id: Some(player_id.into()),
            coordinate: Some(coordinate),
        }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_coordinate(&self) -> Option<FieldCoordinate> { self.coordinate }

    /// Java: `ClientCommandSetupPlayer.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("playerId".to_string(), serde_json::json!(self.player_id));
        if let Some(coordinate) = self.coordinate {
            map.insert("coordinate".to_string(), coordinate.to_json_value());
        }
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandSetupPlayer.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            player_id: json.get("playerId").and_then(|v| v.as_str()).map(|s| s.to_string()),
            coordinate: json.get("coordinate").and_then(FieldCoordinate::from_json),
        }
    }
}

impl NetCommand for ClientCommandSetupPlayer {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientSetupPlayer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored_correctly() {
        let coord = FieldCoordinate::new(5, 5);
        let cmd = ClientCommandSetupPlayer::with_placement("p1", coord);
        assert_eq!(cmd.get_player_id(), Some("p1"));
        assert_eq!(cmd.get_coordinate(), Some(coord));
    }

    #[test]
    fn default_both_none() {
        let cmd = ClientCommandSetupPlayer::new();
        assert!(cmd.player_id.is_none());
        assert!(cmd.coordinate.is_none());
    }

    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandSetupPlayer::new()).is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandSetupPlayer::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandSetupPlayer::default());
        assert!(s.contains("ClientCommandSetupPlayer"));
    }

    #[test]
    fn get_id_is_client_setup_player() {
        assert_eq!(ClientCommandSetupPlayer::new().get_id(), NetCommandId::ClientSetupPlayer);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_player_id() {
        let cmd = ClientCommandSetupPlayer::with_placement("p1", FieldCoordinate::new(5, 5));
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientSetupPlayer");
        assert_eq!(json["playerId"], "p1");
    }

    #[test]
    fn round_trip_with_data() {
        let mut cmd = ClientCommandSetupPlayer::with_placement("p2", FieldCoordinate::new(3, 7));
        cmd.entropy = Some(9);
        let json = cmd.to_json_value();
        let restored = ClientCommandSetupPlayer::from_json(&json);
        assert_eq!(restored.player_id, cmd.player_id);
        assert_eq!(restored.coordinate, cmd.coordinate);
        assert_eq!(restored.entropy, cmd.entropy);
    }

    #[test]
    fn round_trip_default() {
        let cmd = ClientCommandSetupPlayer::default();
        let json = cmd.to_json_value();
        let restored = ClientCommandSetupPlayer::from_json(&json);
        assert!(restored.player_id.is_none());
        assert!(restored.coordinate.is_none());
        assert!(restored.entropy.is_none());
    }
}
