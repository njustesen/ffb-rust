use ffb_model::enums::NetCommandId;
use ffb_model::types::FieldCoordinate;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandTeamSetupSave`.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandTeamSetupSave {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `fSetupName`
    pub setup_name: Option<String>,
    /// Java: `fPlayerNumbers`
    pub player_numbers: Vec<i32>,
    /// Java: `fPlayerCoordinates`
    pub player_coordinates: Vec<FieldCoordinate>,
}

impl ClientCommandTeamSetupSave {
    pub fn new() -> Self { Self::default() }

    pub fn with_setup(
        setup_name: impl Into<String>,
        player_numbers: Vec<i32>,
        player_coordinates: Vec<FieldCoordinate>,
    ) -> Self {
        Self {
            entropy: None,
            setup_name: Some(setup_name.into()),
            player_numbers,
            player_coordinates,
        }
    }

    pub fn get_setup_name(&self) -> Option<&str> { self.setup_name.as_deref() }
    pub fn get_player_numbers(&self) -> &[i32] { &self.player_numbers }
    pub fn get_player_coordinates(&self) -> &[FieldCoordinate] { &self.player_coordinates }

    /// Java: `ClientCommandTeamSetupSave.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("setupName".to_string(), serde_json::json!(self.setup_name));
        map.insert("playerNumbers".to_string(), serde_json::json!(self.player_numbers));
        map.insert(
            "playerCoordinates".to_string(),
            serde_json::Value::Array(self.player_coordinates.iter().map(|fc| fc.to_json_value()).collect()),
        );
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandTeamSetupSave.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        let player_numbers = json
            .get("playerNumbers")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_i64().map(|n| n as i32)).collect())
            .unwrap_or_default();
        let player_coordinates = json
            .get("playerCoordinates")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(FieldCoordinate::from_json).collect())
            .unwrap_or_default();
        Self {
            entropy: base.entropy,
            setup_name: json.get("setupName").and_then(|v| v.as_str()).map(|s| s.to_string()),
            player_numbers,
            player_coordinates,
        }
    }
}

impl NetCommand for ClientCommandTeamSetupSave {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientTeamSetupSave
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let coords = vec![FieldCoordinate::new(1, 2), FieldCoordinate::new(3, 4)];
        let cmd = ClientCommandTeamSetupSave::with_setup("default", vec![1, 2], coords.clone());
        assert_eq!(cmd.get_setup_name(), Some("default"));
        assert_eq!(cmd.get_player_numbers(), &[1, 2]);
        assert_eq!(cmd.get_player_coordinates().len(), 2);
    }

    #[test]
    fn default_is_empty() {
        let cmd = ClientCommandTeamSetupSave::new();
        assert!(cmd.setup_name.is_none());
        assert!(cmd.player_numbers.is_empty());
        assert!(cmd.player_coordinates.is_empty());
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandTeamSetupSave::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandTeamSetupSave::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandTeamSetupSave::default());
        assert!(s.contains("ClientCommandTeamSetupSave"));
    }

    #[test]
    fn get_id_is_client_team_setup_save() {
        assert_eq!(ClientCommandTeamSetupSave::new().get_id(), NetCommandId::ClientTeamSetupSave);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_player_numbers() {
        let cmd = ClientCommandTeamSetupSave::with_setup("s1", vec![1, 2, 3], vec![]);
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientTeamSetupSave");
        assert_eq!(json["playerNumbers"], serde_json::json!([1, 2, 3]));
    }

    #[test]
    fn round_trip_with_all_fields_and_entropy() {
        let coords = vec![FieldCoordinate::new(5, 6), FieldCoordinate::new(7, 8)];
        let mut cmd = ClientCommandTeamSetupSave::with_setup("setup-a", vec![4, 9], coords.clone());
        cmd.entropy = Some(13);
        let json = cmd.to_json_value();
        let restored = ClientCommandTeamSetupSave::from_json(&json);
        assert_eq!(restored.entropy, Some(13));
        assert_eq!(restored.get_setup_name(), Some("setup-a"));
        assert_eq!(restored.get_player_numbers(), &[4, 9]);
        assert_eq!(restored.get_player_coordinates(), coords.as_slice());
    }

    #[test]
    fn round_trip_with_empty_defaults() {
        let cmd = ClientCommandTeamSetupSave::new();
        let json = cmd.to_json_value();
        let restored = ClientCommandTeamSetupSave::from_json(&json);
        assert!(restored.get_setup_name().is_none());
        assert!(restored.get_player_numbers().is_empty());
        assert!(restored.get_player_coordinates().is_empty());
    }
}
