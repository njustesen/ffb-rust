use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of ClientCommandPickUpChoice (Java field: attemptPickUp).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandPickUpChoice {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    pub attempt_pick_up: bool,
}

impl ClientCommandPickUpChoice {
    pub fn new(attempt_pick_up: bool) -> Self {
        Self { entropy: None, attempt_pick_up }
    }

    pub fn is_attempt_pick_up(&self) -> bool {
        self.attempt_pick_up
    }

    /// Java: `ClientCommandPickUpChoice.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("choicePickUp".to_string(), serde_json::json!(self.attempt_pick_up));
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandPickUpChoice.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            attempt_pick_up: json.get("choicePickUp").and_then(|v| v.as_bool()).unwrap_or(false),
        }
    }
}

impl NetCommand for ClientCommandPickUpChoice {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientPickUpChoice
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_true_stores_true() {
        let cmd = ClientCommandPickUpChoice::new(true);
        assert!(cmd.is_attempt_pick_up());
    }

    #[test]
    fn new_false_stores_false() {
        let cmd = ClientCommandPickUpChoice::new(false);
        assert!(!cmd.is_attempt_pick_up());
    }

    #[test]
    fn default_is_false() {
        let cmd = ClientCommandPickUpChoice::default();
        assert!(!cmd.is_attempt_pick_up());
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandPickUpChoice::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandPickUpChoice::default().clone();
    }

    #[test]
    fn get_id_is_client_pick_up_choice() {
        assert_eq!(ClientCommandPickUpChoice::default().get_id(), NetCommandId::ClientPickUpChoice);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_choice_pick_up() {
        let cmd = ClientCommandPickUpChoice::new(true);
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientPickUpChoice");
        assert_eq!(json["choicePickUp"], true);
    }

    #[test]
    fn round_trip_with_data() {
        let mut cmd = ClientCommandPickUpChoice::new(true);
        cmd.entropy = Some(5);
        let json = cmd.to_json_value();
        let restored = ClientCommandPickUpChoice::from_json(&json);
        assert_eq!(restored.entropy, Some(5));
        assert!(restored.is_attempt_pick_up());
    }

    #[test]
    fn round_trip_default() {
        let cmd = ClientCommandPickUpChoice::default();
        let json = cmd.to_json_value();
        let restored = ClientCommandPickUpChoice::from_json(&json);
        assert!(!restored.is_attempt_pick_up());
        assert!(restored.entropy.is_none());
    }
}
