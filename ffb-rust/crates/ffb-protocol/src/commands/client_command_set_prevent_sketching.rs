use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;
use ffb_model::enums::NetCommandId;

/// 1:1 translation of ClientCommandSetPreventSketching (Java fields: coach, preventSketching).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandSetPreventSketching {
    pub coach: Option<String>,
    pub prevent_sketching: bool,
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
}

impl ClientCommandSetPreventSketching {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_fields(coach: impl Into<String>, prevent_sketching: bool) -> Self {
        Self {
            coach: Some(coach.into()),
            prevent_sketching,
            entropy: None,
        }
    }

    pub fn get_coach(&self) -> Option<&str> {
        self.coach.as_deref()
    }

    pub fn is_prevent_sketching(&self) -> bool {
        self.prevent_sketching
    }

    /// Java: `ClientCommandSetPreventSketching.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("prevent".to_string(), serde_json::json!(self.prevent_sketching));
        map.insert("coach".to_string(), match &self.coach {
            Some(s) => serde_json::json!(s),
            None => serde_json::Value::Null,
        });
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandSetPreventSketching.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            prevent_sketching: json.get("prevent").and_then(|v| v.as_bool()).unwrap_or(false),
            coach: json.get("coach").and_then(|v| v.as_str()).map(|s| s.to_string()),
            entropy: base.entropy,
        }
    }
}

impl NetCommand for ClientCommandSetPreventSketching {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientSetPreventSketching
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_no_coach_and_false_flag() {
        let cmd = ClientCommandSetPreventSketching::new();
        assert!(cmd.get_coach().is_none());
        assert!(!cmd.is_prevent_sketching());
    }

    #[test]
    fn with_fields_stores_values() {
        let cmd = ClientCommandSetPreventSketching::with_fields("coach-1", true);
        assert_eq!(cmd.get_coach(), Some("coach-1"));
        assert!(cmd.is_prevent_sketching());
    }

    #[test]
    fn false_prevent_stored() {
        let cmd = ClientCommandSetPreventSketching::with_fields("c", false);
        assert!(!cmd.is_prevent_sketching());
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandSetPreventSketching::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandSetPreventSketching::default().clone();
    }

    #[test]
    fn get_id_is_client_set_prevent_sketching() {
        assert_eq!(
            ClientCommandSetPreventSketching::new().get_id(),
            NetCommandId::ClientSetPreventSketching
        );
    }

    #[test]
    fn to_json_value_has_net_command_id_and_prevent() {
        let cmd = ClientCommandSetPreventSketching::with_fields("coach-1", true);
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientSetPreventSketching");
        assert_eq!(json["prevent"], true);
        assert_eq!(json["coach"], "coach-1");
    }

    #[test]
    fn round_trip_populated() {
        let mut cmd = ClientCommandSetPreventSketching::with_fields("coach-2", true);
        cmd.entropy = Some(1);
        let json = cmd.to_json_value();
        let restored = ClientCommandSetPreventSketching::from_json(&json);
        assert_eq!(restored.coach.as_deref(), Some("coach-2"));
        assert!(restored.prevent_sketching);
        assert_eq!(restored.entropy, Some(1));
    }

    #[test]
    fn round_trip_default() {
        let cmd = ClientCommandSetPreventSketching::default();
        let json = cmd.to_json_value();
        let restored = ClientCommandSetPreventSketching::from_json(&json);
        assert_eq!(restored.coach, None);
        assert!(!restored.prevent_sketching);
        assert_eq!(restored.entropy, None);
    }
}
