use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of ClientCommandTeamSetupDelete (Java field: fSetupName).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandTeamSetupDelete {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    pub setup_name: Option<String>,
}

impl ClientCommandTeamSetupDelete {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_setup_name(name: impl Into<String>) -> Self {
        Self { entropy: None, setup_name: Some(name.into()) }
    }

    pub fn get_setup_name(&self) -> Option<&str> {
        self.setup_name.as_deref()
    }

    /// Java: `ClientCommandTeamSetupDelete.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("setupName".to_string(), serde_json::json!(self.setup_name));
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandTeamSetupDelete.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            setup_name: json.get("setupName").and_then(|v| v.as_str()).map(|s| s.to_string()),
        }
    }
}

impl NetCommand for ClientCommandTeamSetupDelete {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientTeamSetupDelete
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_no_setup_name() {
        let cmd = ClientCommandTeamSetupDelete::new();
        assert!(cmd.get_setup_name().is_none());
    }

    #[test]
    fn with_setup_name_stores_value() {
        let cmd = ClientCommandTeamSetupDelete::with_setup_name("my-setup");
        assert_eq!(cmd.get_setup_name(), Some("my-setup"));
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandTeamSetupDelete::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandTeamSetupDelete::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandTeamSetupDelete::default());
        assert!(s.contains("ClientCommandTeamSetupDelete"));
    }

    #[test]
    fn get_id_is_client_team_setup_delete() {
        assert_eq!(ClientCommandTeamSetupDelete::new().get_id(), NetCommandId::ClientTeamSetupDelete);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_setup_name() {
        let cmd = ClientCommandTeamSetupDelete::with_setup_name("s1");
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientTeamSetupDelete");
        assert_eq!(json["setupName"], "s1");
    }

    #[test]
    fn round_trip_with_setup_name_and_entropy() {
        let mut cmd = ClientCommandTeamSetupDelete::with_setup_name("s2");
        cmd.entropy = Some(11);
        let json = cmd.to_json_value();
        let restored = ClientCommandTeamSetupDelete::from_json(&json);
        assert_eq!(restored.entropy, Some(11));
        assert_eq!(restored.get_setup_name(), Some("s2"));
    }

    #[test]
    fn round_trip_with_no_setup_name() {
        let cmd = ClientCommandTeamSetupDelete::new();
        let json = cmd.to_json_value();
        let restored = ClientCommandTeamSetupDelete::from_json(&json);
        assert!(restored.get_setup_name().is_none());
    }
}
