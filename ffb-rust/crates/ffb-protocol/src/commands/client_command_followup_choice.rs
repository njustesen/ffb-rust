/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandFollowupChoice`.
/// Sent when the attacker decides whether to follow up after a block pushback.
use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

#[derive(Debug, Clone, Default)]
pub struct ClientCommandFollowupChoice {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `fChoiceFollowup`
    pub choice_followup: bool,
}

impl ClientCommandFollowupChoice {
    pub fn new(choice_followup: bool) -> Self {
        Self { entropy: None, choice_followup }
    }

    /// Java: `isChoiceFollowup()`
    pub fn is_choice_followup(&self) -> bool { self.choice_followup }

    /// Java: `ClientCommandFollowupChoice.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("choiceFollowup".to_string(), serde_json::json!(self.choice_followup));
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandFollowupChoice.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            choice_followup: json.get("choiceFollowup").and_then(|v| v.as_bool()).unwrap_or(false),
        }
    }
}

impl NetCommand for ClientCommandFollowupChoice {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientFollowupChoice
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn followup_true_stored() {
        let cmd = ClientCommandFollowupChoice::new(true);
        assert!(cmd.is_choice_followup());
    }

    #[test]
    fn followup_false_stored() {
        let cmd = ClientCommandFollowupChoice::new(false);
        assert!(!cmd.is_choice_followup());
    }

    #[test]
    fn default_no_followup() {
        let cmd = ClientCommandFollowupChoice::default();
        assert!(!cmd.choice_followup);
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandFollowupChoice::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandFollowupChoice::default().clone();
    }

    #[test]
    fn get_id_is_client_followup_choice() {
        assert_eq!(ClientCommandFollowupChoice::new(true).get_id(), NetCommandId::ClientFollowupChoice);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_choice_followup() {
        let cmd = ClientCommandFollowupChoice::new(true);
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientFollowupChoice");
        assert_eq!(json["choiceFollowup"], true);
    }

    #[test]
    fn round_trip_with_entropy() {
        let mut cmd = ClientCommandFollowupChoice::new(true);
        cmd.entropy = Some(2);
        let json = cmd.to_json_value();
        let restored = ClientCommandFollowupChoice::from_json(&json);
        assert_eq!(restored.entropy, Some(2));
        assert!(restored.choice_followup);
    }

    #[test]
    fn round_trip_default() {
        let cmd = ClientCommandFollowupChoice::default();
        let json = cmd.to_json_value();
        let restored = ClientCommandFollowupChoice::from_json(&json);
        assert!(!restored.choice_followup);
        assert!(restored.entropy.is_none());
    }
}
