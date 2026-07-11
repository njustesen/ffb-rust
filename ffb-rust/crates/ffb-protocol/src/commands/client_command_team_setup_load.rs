use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of ClientCommandTeamSetupLoad (Java field: fSetupName).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandTeamSetupLoad {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    pub setup_name: Option<String>,
}

impl ClientCommandTeamSetupLoad {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_setup_name(name: impl Into<String>) -> Self {
        Self { entropy: None, setup_name: Some(name.into()) }
    }

    pub fn get_setup_name(&self) -> Option<&str> {
        self.setup_name.as_deref()
    }

    /// Java: `ClientCommandTeamSetupLoad.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("setupName".to_string(), serde_json::json!(self.setup_name));
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandTeamSetupLoad.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            setup_name: json.get("setupName").and_then(|v| v.as_str()).map(|s| s.to_string()),
        }
    }
}

impl NetCommand for ClientCommandTeamSetupLoad {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientTeamSetupLoad
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_no_setup_name() {
        let cmd = ClientCommandTeamSetupLoad::new();
        assert!(cmd.get_setup_name().is_none());
    }

    #[test]
    fn with_setup_name_stores_value() {
        let cmd = ClientCommandTeamSetupLoad::with_setup_name("default-setup");
        assert_eq!(cmd.get_setup_name(), Some("default-setup"));
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandTeamSetupLoad::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandTeamSetupLoad::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandTeamSetupLoad::default());
        assert!(s.contains("ClientCommandTeamSetupLoad"));
    }

    #[test]
    fn get_id_is_client_team_setup_load() {
        assert_eq!(ClientCommandTeamSetupLoad::new().get_id(), NetCommandId::ClientTeamSetupLoad);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_setup_name() {
        let cmd = ClientCommandTeamSetupLoad::with_setup_name("s1");
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientTeamSetupLoad");
        assert_eq!(json["setupName"], "s1");
    }

    #[test]
    fn round_trip_with_setup_name_and_entropy() {
        let mut cmd = ClientCommandTeamSetupLoad::with_setup_name("s2");
        cmd.entropy = Some(11);
        let json = cmd.to_json_value();
        let restored = ClientCommandTeamSetupLoad::from_json(&json);
        assert_eq!(restored.entropy, Some(11));
        assert_eq!(restored.get_setup_name(), Some("s2"));
    }

    #[test]
    fn round_trip_with_no_setup_name() {
        let cmd = ClientCommandTeamSetupLoad::new();
        let json = cmd.to_json_value();
        let restored = ClientCommandTeamSetupLoad::from_json(&json);
        assert!(restored.get_setup_name().is_none());
    }
}
