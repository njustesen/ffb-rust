use ffb_model::enums::NetCommandId;
use ffb_model::types::FieldCoordinate;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandTouchback`.
/// Sent when a touchback occurs — the receiving team places the ball.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandTouchback {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `fBallCoordinate`
    pub ball_coordinate: Option<FieldCoordinate>,
}

impl ClientCommandTouchback {
    pub fn new(ball_coordinate: FieldCoordinate) -> Self {
        Self { entropy: None, ball_coordinate: Some(ball_coordinate) }
    }

    /// Java: `getBallCoordinate()`
    pub fn get_ball_coordinate(&self) -> Option<FieldCoordinate> {
        self.ball_coordinate
    }

    /// Java: `ClientCommandTouchback.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert(
            "ballCoordinate".to_string(),
            self.ball_coordinate.map(|fc| fc.to_json_value()).unwrap_or(serde_json::Value::Null),
        );
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandTouchback.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            ball_coordinate: json.get("ballCoordinate").and_then(FieldCoordinate::from_json),
        }
    }
}

impl NetCommand for ClientCommandTouchback {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientTouchback
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coordinate_stored() {
        let coord = FieldCoordinate::new(12, 8);
        let cmd = ClientCommandTouchback::new(coord);
        assert_eq!(cmd.get_ball_coordinate(), Some(coord));
    }

    #[test]
    fn default_is_empty() {
        let cmd = ClientCommandTouchback::default();
        assert!(cmd.ball_coordinate.is_none());
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandTouchback::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandTouchback::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandTouchback::default());
        assert!(s.contains("ClientCommandTouchback"));
    }

    #[test]
    fn get_id_is_client_touchback() {
        assert_eq!(ClientCommandTouchback::default().get_id(), NetCommandId::ClientTouchback);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_ball_coordinate() {
        let cmd = ClientCommandTouchback::new(FieldCoordinate::new(3, 4));
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientTouchback");
        assert_eq!(json["ballCoordinate"], serde_json::json!([3, 4]));
    }

    #[test]
    fn round_trip_with_coordinate_and_entropy() {
        let mut cmd = ClientCommandTouchback::new(FieldCoordinate::new(5, 6));
        cmd.entropy = Some(8);
        let json = cmd.to_json_value();
        let restored = ClientCommandTouchback::from_json(&json);
        assert_eq!(restored.entropy, Some(8));
        assert_eq!(restored.get_ball_coordinate(), Some(FieldCoordinate::new(5, 6)));
    }

    #[test]
    fn round_trip_with_no_coordinate() {
        let cmd = ClientCommandTouchback::default();
        let json = cmd.to_json_value();
        let restored = ClientCommandTouchback::from_json(&json);
        assert!(restored.get_ball_coordinate().is_none());
    }
}
