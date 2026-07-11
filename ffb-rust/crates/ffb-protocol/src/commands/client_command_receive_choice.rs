use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandReceiveChoice`.
/// Sent when the coach chooses to receive or kick at the start of a half.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandReceiveChoice {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `fChoiceReceive`
    pub choice_receive: bool,
}

impl ClientCommandReceiveChoice {
    pub fn new(choice_receive: bool) -> Self {
        Self { entropy: None, choice_receive }
    }

    /// Java: `isChoiceReceive()`
    pub fn is_choice_receive(&self) -> bool { self.choice_receive }

    /// Java: `ClientCommandReceiveChoice.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("choiceReceive".to_string(), serde_json::json!(self.choice_receive));
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandReceiveChoice.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            choice_receive: json.get("choiceReceive").and_then(|v| v.as_bool()).unwrap_or(false),
        }
    }
}

impl NetCommand for ClientCommandReceiveChoice {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientReceiveChoice
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn receive_true_stored() {
        let cmd = ClientCommandReceiveChoice::new(true);
        assert!(cmd.is_choice_receive());
    }

    #[test]
    fn kick_stored() {
        let cmd = ClientCommandReceiveChoice::new(false);
        assert!(!cmd.is_choice_receive());
    }

    #[test]
    fn default_is_kick() {
        let cmd = ClientCommandReceiveChoice::default();
        assert!(!cmd.choice_receive);
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandReceiveChoice::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandReceiveChoice::default().clone();
    }

    #[test]
    fn get_id_is_client_receive_choice() {
        assert_eq!(ClientCommandReceiveChoice::default().get_id(), NetCommandId::ClientReceiveChoice);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_choice_receive() {
        let cmd = ClientCommandReceiveChoice::new(true);
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientReceiveChoice");
        assert_eq!(json["choiceReceive"], true);
    }

    #[test]
    fn round_trip_with_data() {
        let mut cmd = ClientCommandReceiveChoice::new(true);
        cmd.entropy = Some(7);
        let json = cmd.to_json_value();
        let restored = ClientCommandReceiveChoice::from_json(&json);
        assert_eq!(restored.entropy, Some(7));
        assert!(restored.is_choice_receive());
    }

    #[test]
    fn round_trip_default() {
        let cmd = ClientCommandReceiveChoice::default();
        let json = cmd.to_json_value();
        let restored = ClientCommandReceiveChoice::from_json(&json);
        assert!(!restored.is_choice_receive());
        assert!(restored.entropy.is_none());
    }
}
