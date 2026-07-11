use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of ClientCommandPasswordChallenge (Java field: fCoach).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandPasswordChallenge {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    pub coach: Option<String>,
}

impl ClientCommandPasswordChallenge {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_coach(coach: impl Into<String>) -> Self {
        Self { entropy: None, coach: Some(coach.into()) }
    }

    pub fn get_coach(&self) -> Option<&str> {
        self.coach.as_deref()
    }

    /// Java: `ClientCommandPasswordChallenge.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        if let Some(coach) = &self.coach {
            map.insert("coach".to_string(), serde_json::json!(coach));
        }
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandPasswordChallenge.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            coach: json.get("coach").and_then(|v| v.as_str()).map(String::from),
        }
    }
}

impl NetCommand for ClientCommandPasswordChallenge {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientPasswordChallenge
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_no_coach() {
        let cmd = ClientCommandPasswordChallenge::new();
        assert!(cmd.get_coach().is_none());
    }

    #[test]
    fn with_coach_stores_value() {
        let cmd = ClientCommandPasswordChallenge::with_coach("coach-xyz");
        assert_eq!(cmd.get_coach(), Some("coach-xyz"));
    }

    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandPasswordChallenge::new()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandPasswordChallenge::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandPasswordChallenge::default());
        assert!(s.contains("ClientCommandPasswordChallenge"));
    }

    #[test]
    fn get_id_is_client_password_challenge() {
        assert_eq!(ClientCommandPasswordChallenge::new().get_id(), NetCommandId::ClientPasswordChallenge);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_coach() {
        let cmd = ClientCommandPasswordChallenge::with_coach("coach1");
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientPasswordChallenge");
        assert_eq!(json["coach"], "coach1");
    }

    #[test]
    fn round_trip_with_data() {
        let mut cmd = ClientCommandPasswordChallenge::with_coach("coach2");
        cmd.entropy = Some(11);
        let json = cmd.to_json_value();
        let restored = ClientCommandPasswordChallenge::from_json(&json);
        assert_eq!(restored.entropy, Some(11));
        assert_eq!(restored.get_coach(), Some("coach2"));
    }

    #[test]
    fn round_trip_default() {
        let cmd = ClientCommandPasswordChallenge::default();
        let json = cmd.to_json_value();
        let restored = ClientCommandPasswordChallenge::from_json(&json);
        assert!(restored.coach.is_none());
        assert!(restored.entropy.is_none());
    }
}
