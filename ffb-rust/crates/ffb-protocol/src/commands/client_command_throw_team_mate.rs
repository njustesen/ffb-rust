use ffb_model::enums::NetCommandId;
use ffb_model::types::FieldCoordinate;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandThrowTeamMate`.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandThrowTeamMate {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `fTargetCoordinate`
    pub target_coordinate: Option<FieldCoordinate>,
    /// Java: `fThrownPlayerId`
    pub thrown_player_id: Option<String>,
    /// Java: `fActingPlayerId`
    pub acting_player_id: Option<String>,
    /// Java: `kicked`
    pub kicked: bool,
}

impl ClientCommandThrowTeamMate {
    pub fn new(
        target_coordinate: FieldCoordinate,
        thrown_player_id: impl Into<String>,
        acting_player_id: impl Into<String>,
        kicked: bool,
    ) -> Self {
        Self {
            entropy: None,
            target_coordinate: Some(target_coordinate),
            thrown_player_id: Some(thrown_player_id.into()),
            acting_player_id: Some(acting_player_id.into()),
            kicked,
        }
    }

    pub fn get_target_coordinate(&self) -> Option<FieldCoordinate> { self.target_coordinate }
    pub fn get_thrown_player_id(&self) -> Option<&str> { self.thrown_player_id.as_deref() }
    pub fn get_acting_player_id(&self) -> Option<&str> { self.acting_player_id.as_deref() }
    pub fn is_kicked(&self) -> bool { self.kicked }

    /// Java: `ClientCommandThrowTeamMate.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("actingPlayerId".to_string(), serde_json::json!(self.acting_player_id));
        map.insert("thrownPlayerId".to_string(), serde_json::json!(self.thrown_player_id));
        map.insert(
            "targetCoordinate".to_string(),
            self.target_coordinate.map(|fc| fc.to_json_value()).unwrap_or(serde_json::Value::Null),
        );
        map.insert("kicked".to_string(), serde_json::json!(self.kicked));
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandThrowTeamMate.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            acting_player_id: json.get("actingPlayerId").and_then(|v| v.as_str()).map(|s| s.to_string()),
            thrown_player_id: json.get("thrownPlayerId").and_then(|v| v.as_str()).map(|s| s.to_string()),
            target_coordinate: json.get("targetCoordinate").and_then(FieldCoordinate::from_json),
            kicked: json.get("kicked").and_then(|v| v.as_bool()).unwrap_or(false),
        }
    }
}

impl NetCommand for ClientCommandThrowTeamMate {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientThrowTeamMate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_fields_stored() {
        let coord = FieldCoordinate::new(5, 8);
        let cmd = ClientCommandThrowTeamMate::new(coord, "thrown1", "thrower1", true);
        assert_eq!(cmd.get_target_coordinate(), Some(coord));
        assert_eq!(cmd.get_thrown_player_id(), Some("thrown1"));
        assert_eq!(cmd.get_acting_player_id(), Some("thrower1"));
        assert!(cmd.is_kicked());
    }

    #[test]
    fn default_is_empty() {
        let cmd = ClientCommandThrowTeamMate::default();
        assert!(cmd.target_coordinate.is_none());
        assert!(cmd.thrown_player_id.is_none());
        assert!(!cmd.kicked);
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandThrowTeamMate::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandThrowTeamMate::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandThrowTeamMate::default());
        assert!(s.contains("ClientCommandThrowTeamMate"));
    }

    #[test]
    fn get_id_is_client_throw_team_mate() {
        assert_eq!(ClientCommandThrowTeamMate::default().get_id(), NetCommandId::ClientThrowTeamMate);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_kicked() {
        let cmd = ClientCommandThrowTeamMate::new(FieldCoordinate::new(1, 1), "t", "a", true);
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientThrowTeamMate");
        assert_eq!(json["kicked"], true);
    }

    #[test]
    fn round_trip_with_all_fields_and_entropy() {
        let coord = FieldCoordinate::new(9, 10);
        let mut cmd = ClientCommandThrowTeamMate::new(coord, "thrown2", "thrower2", false);
        cmd.entropy = Some(21);
        let json = cmd.to_json_value();
        let restored = ClientCommandThrowTeamMate::from_json(&json);
        assert_eq!(restored.entropy, Some(21));
        assert_eq!(restored.get_target_coordinate(), Some(coord));
        assert_eq!(restored.get_thrown_player_id(), Some("thrown2"));
        assert_eq!(restored.get_acting_player_id(), Some("thrower2"));
        assert!(!restored.is_kicked());
    }

    #[test]
    fn round_trip_with_no_coordinate_or_ids() {
        let cmd = ClientCommandThrowTeamMate::default();
        let json = cmd.to_json_value();
        let restored = ClientCommandThrowTeamMate::from_json(&json);
        assert!(restored.get_target_coordinate().is_none());
        assert!(restored.get_thrown_player_id().is_none());
        assert!(restored.get_acting_player_id().is_none());
        assert!(!restored.is_kicked());
    }
}
