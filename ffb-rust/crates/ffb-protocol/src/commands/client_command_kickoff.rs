use ffb_model::enums::NetCommandId;
use ffb_model::types::FieldCoordinate;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandKickoff`.
/// Sent by the kicking team to place the ball for kickoff.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandKickoff {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `fBallCoordinate`
    pub ball_coordinate: Option<FieldCoordinate>,
}

impl ClientCommandKickoff {
    pub fn new(ball_coordinate: FieldCoordinate) -> Self {
        Self { entropy: None, ball_coordinate: Some(ball_coordinate) }
    }

    /// Java: `getBallCoordinate()`
    pub fn get_ball_coordinate(&self) -> Option<FieldCoordinate> {
        self.ball_coordinate
    }

    /// Java: `ClientCommandKickoff.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        if let Some(ball_coordinate) = self.ball_coordinate {
            map.insert("ballCoordinate".to_string(), ball_coordinate.to_json_value());
        }
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandKickoff.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            ball_coordinate: json.get("ballCoordinate").and_then(FieldCoordinate::from_json),
        }
    }
}

impl NetCommand for ClientCommandKickoff {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientKickoff
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coordinate_stored() {
        let coord = FieldCoordinate::new(7, 3);
        let cmd = ClientCommandKickoff::new(coord);
        assert_eq!(cmd.get_ball_coordinate(), Some(coord));
    }

    #[test]
    fn default_has_no_coordinate() {
        let cmd = ClientCommandKickoff::default();
        assert!(cmd.ball_coordinate.is_none());
    }

    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandKickoff::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandKickoff::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandKickoff::default());
        assert!(s.contains("ClientCommandKickoff"));
    }

    #[test]
    fn get_id_is_client_kickoff() {
        assert_eq!(ClientCommandKickoff::default().get_id(), NetCommandId::ClientKickoff);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_ball_coordinate() {
        let cmd = ClientCommandKickoff::new(FieldCoordinate::new(2, 9));
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientKickoff");
        assert_eq!(json["ballCoordinate"], serde_json::json!([2, 9]));
    }

    #[test]
    fn round_trip_with_coordinate_and_entropy() {
        let mut cmd = ClientCommandKickoff::new(FieldCoordinate::new(5, 6));
        cmd.entropy = Some(2);
        let json = cmd.to_json_value();
        let restored = ClientCommandKickoff::from_json(&json);
        assert_eq!(restored.entropy, Some(2));
        assert_eq!(restored.get_ball_coordinate(), Some(FieldCoordinate::new(5, 6)));
    }

    #[test]
    fn round_trip_with_no_coordinate() {
        let cmd = ClientCommandKickoff::default();
        let json = cmd.to_json_value();
        let restored = ClientCommandKickoff::from_json(&json);
        assert!(restored.get_ball_coordinate().is_none());
    }
}
