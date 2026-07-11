use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;
use ffb_model::enums::NetCommandId;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandBloodlustAction`.
/// Sent when a vampire player chooses to change their blood lust action.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandBloodlustAction {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `change` — whether the player wants to change their action.
    pub change: bool,
}

impl ClientCommandBloodlustAction {
    pub fn new(change: bool) -> Self { Self { entropy: None, change } }
    pub fn is_change(&self) -> bool { self.change }

    /// Java: `ClientCommandBloodlustAction.toJsonValue()` (calls `super.toJsonValue()` first).
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("changeToMove".to_string(), serde_json::json!(self.change));
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandBloodlustAction.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            change: json.get("changeToMove").and_then(|v| v.as_bool()).unwrap_or(false),
        }
    }
}

impl NetCommand for ClientCommandBloodlustAction {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientBloodlustAction
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn change_true_stored() {
        let cmd = ClientCommandBloodlustAction::new(true);
        assert!(cmd.is_change());
    }

    #[test]
    fn default_is_false() {
        let cmd = ClientCommandBloodlustAction::default();
        assert!(!cmd.change);
    }

    #[test]
    fn change_false_stored() {
        let cmd = ClientCommandBloodlustAction::new(false);
        assert!(!cmd.is_change());
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandBloodlustAction::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandBloodlustAction::default().clone();
    }

    #[test]
    fn get_id_is_client_bloodlust_action() {
        assert_eq!(ClientCommandBloodlustAction::new(false).get_id(), NetCommandId::ClientBloodlustAction);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_change_to_move() {
        let cmd = ClientCommandBloodlustAction::new(true);
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientBloodlustAction");
        assert_eq!(json["changeToMove"], true);
    }

    #[test]
    fn round_trip_with_change_true_and_entropy() {
        let mut cmd = ClientCommandBloodlustAction::new(true);
        cmd.entropy = Some(5);
        let json = cmd.to_json_value();
        let restored = ClientCommandBloodlustAction::from_json(&json);
        assert_eq!(restored.entropy, Some(5));
        assert!(restored.is_change());
    }

    #[test]
    fn round_trip_with_default() {
        let cmd = ClientCommandBloodlustAction::default();
        let json = cmd.to_json_value();
        let restored = ClientCommandBloodlustAction::from_json(&json);
        assert_eq!(restored.entropy, None);
        assert!(!restored.is_change());
    }
}
