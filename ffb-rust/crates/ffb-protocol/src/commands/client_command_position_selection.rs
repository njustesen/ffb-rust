use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandPositionSelection`.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandPositionSelection {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `position` (String[])
    pub position: Vec<String>,
    /// Java: `teamId`
    pub team_id: Option<String>,
}

impl ClientCommandPositionSelection {
    pub fn new() -> Self { Self::default() }

    pub fn with_team(team_id: impl Into<String>, position: Vec<String>) -> Self {
        Self { entropy: None, position, team_id: Some(team_id.into()) }
    }

    pub fn get_position(&self) -> &[String] { &self.position }
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }

    /// Java: `ClientCommandPositionSelection.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("positionIds".to_string(), serde_json::json!(self.position));
        if let Some(team_id) = &self.team_id {
            map.insert("teamId".to_string(), serde_json::json!(team_id));
        }
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandPositionSelection.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            position: json
                .get("positionIds")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default(),
            team_id: json.get("teamId").and_then(|v| v.as_str()).map(String::from),
        }
    }
}

impl NetCommand for ClientCommandPositionSelection {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientPositionSelection
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let cmd = ClientCommandPositionSelection::with_team(
            "team1",
            vec!["Lineman".to_string(), "Blitzer".to_string()],
        );
        assert_eq!(cmd.get_team_id(), Some("team1"));
        assert_eq!(cmd.get_position().len(), 2);
    }

    #[test]
    fn default_is_empty() {
        let cmd = ClientCommandPositionSelection::new();
        assert!(cmd.team_id.is_none());
        assert!(cmd.position.is_empty());
    }

    #[test]
    fn position_slice_matches_input() {
        let positions = vec!["Lineman".to_string()];
        let cmd = ClientCommandPositionSelection::with_team("t1", positions.clone());
        assert_eq!(cmd.get_position(), positions.as_slice());
    }


    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandPositionSelection::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandPositionSelection::default().clone();
    }

    #[test]
    fn get_id_is_client_position_selection() {
        assert_eq!(ClientCommandPositionSelection::new().get_id(), NetCommandId::ClientPositionSelection);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_position_ids() {
        let cmd = ClientCommandPositionSelection::with_team("t1", vec!["Blitzer".to_string()]);
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientPositionSelection");
        assert_eq!(json["positionIds"], serde_json::json!(["Blitzer"]));
        assert_eq!(json["teamId"], "t1");
    }

    #[test]
    fn round_trip_with_data() {
        let mut cmd = ClientCommandPositionSelection::with_team("t2", vec!["Lineman".to_string(), "Ogre".to_string()]);
        cmd.entropy = Some(1);
        let json = cmd.to_json_value();
        let restored = ClientCommandPositionSelection::from_json(&json);
        assert_eq!(restored.entropy, Some(1));
        assert_eq!(restored.get_team_id(), Some("t2"));
        assert_eq!(restored.get_position(), &["Lineman".to_string(), "Ogre".to_string()]);
    }

    #[test]
    fn round_trip_default() {
        let cmd = ClientCommandPositionSelection::default();
        let json = cmd.to_json_value();
        let restored = ClientCommandPositionSelection::from_json(&json);
        assert!(restored.position.is_empty());
        assert!(restored.team_id.is_none());
        assert!(restored.entropy.is_none());
    }
}
