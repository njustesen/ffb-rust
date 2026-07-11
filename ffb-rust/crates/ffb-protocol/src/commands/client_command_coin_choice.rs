use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;
use ffb_model::enums::NetCommandId;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandCoinChoice`.
/// Sent by a client when choosing heads or tails for the coin flip.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandCoinChoice {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `fChoiceHeads`
    pub choice_heads: bool,
}

impl ClientCommandCoinChoice {
    pub fn new(choice_heads: bool) -> Self {
        Self { entropy: None, choice_heads }
    }

    /// Java: `isChoiceHeads()`
    pub fn is_choice_heads(&self) -> bool {
        self.choice_heads
    }

    /// Java: `ClientCommandCoinChoice.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("choiceHeads".to_string(), serde_json::json!(self.choice_heads));
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandCoinChoice.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            choice_heads: json.get("choiceHeads").and_then(|v| v.as_bool()).unwrap_or(false),
        }
    }
}

impl NetCommand for ClientCommandCoinChoice {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientCoinChoice
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn choice_heads_stored() {
        let cmd = ClientCommandCoinChoice::new(true);
        assert!(cmd.is_choice_heads());
    }

    #[test]
    fn choice_tails_stored() {
        let cmd = ClientCommandCoinChoice::new(false);
        assert!(!cmd.is_choice_heads());
    }

    #[test]
    fn default_is_tails() {
        let cmd = ClientCommandCoinChoice::default();
        assert!(!cmd.choice_heads);
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandCoinChoice::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandCoinChoice::default().clone();
    }

    #[test]
    fn get_id_is_client_coin_choice() {
        assert_eq!(ClientCommandCoinChoice::new(true).get_id(), NetCommandId::ClientCoinChoice);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_choice_heads() {
        let cmd = ClientCommandCoinChoice::new(true);
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientCoinChoice");
        assert_eq!(json["choiceHeads"], true);
    }

    #[test]
    fn round_trip_with_entropy() {
        let mut cmd = ClientCommandCoinChoice::new(true);
        cmd.entropy = Some(3);
        let json = cmd.to_json_value();
        let restored = ClientCommandCoinChoice::from_json(&json);
        assert_eq!(restored.entropy, Some(3));
        assert!(restored.choice_heads);
    }

    #[test]
    fn round_trip_with_default() {
        let cmd = ClientCommandCoinChoice::default();
        let json = cmd.to_json_value();
        let restored = ClientCommandCoinChoice::from_json(&json);
        assert!(restored.entropy.is_none());
        assert!(!restored.choice_heads);
    }
}
