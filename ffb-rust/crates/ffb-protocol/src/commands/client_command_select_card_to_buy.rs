use ffb_model::enums::NetCommandId;
use crate::net_command::NetCommand;
use crate::commands::client_command::ClientCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandSelectCardToBuy`.
/// Java has an inner `Selection` class with `initialDeckChoice` and `firstCardChoice` booleans;
/// simplified here to two top-level fields.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandSelectCardToBuy {
    /// Java: `selection.initialDeckChoice`
    pub initial_deck_choice: bool,
    /// Java: `selection.firstCardChoice`
    pub first_card_choice: bool,
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
}

impl ClientCommandSelectCardToBuy {
    pub fn new(initial_deck_choice: bool, first_card_choice: bool) -> Self {
        Self { initial_deck_choice, first_card_choice, entropy: None }
    }

    pub fn is_initial_deck_choice(&self) -> bool { self.initial_deck_choice }
    pub fn is_first_card_choice(&self) -> bool { self.first_card_choice }

    /// Java: `Selection.name()` — reconstructed from the two flattened booleans
    /// (`INITIAL_FIRST`, `INITIAL_SECOND`, `REROLLED_FIRST`, `REROLLED_SECOND`).
    fn selection_name(&self) -> &'static str {
        match (self.initial_deck_choice, self.first_card_choice) {
            (true, true) => "INITIAL_FIRST",
            (true, false) => "INITIAL_SECOND",
            (false, true) => "REROLLED_FIRST",
            (false, false) => "REROLLED_SECOND",
        }
    }

    /// Java: `ClientCommandSelectCardToBuy.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("cardSelection".to_string(), serde_json::json!(self.selection_name()));
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandSelectCardToBuy.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        let (initial_deck_choice, first_card_choice) = match json.get("cardSelection").and_then(|v| v.as_str()) {
            Some("INITIAL_FIRST") => (true, true),
            Some("INITIAL_SECOND") => (true, false),
            Some("REROLLED_FIRST") => (false, true),
            Some("REROLLED_SECOND") => (false, false),
            _ => (false, false),
        };
        Self {
            initial_deck_choice,
            first_card_choice,
            entropy: base.entropy,
        }
    }
}

impl NetCommand for ClientCommandSelectCardToBuy {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientSelectCardToBuy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bools_stored() {
        let cmd = ClientCommandSelectCardToBuy::new(true, false);
        assert!(cmd.is_initial_deck_choice());
        assert!(!cmd.is_first_card_choice());
    }

    #[test]
    fn default_is_false() {
        let cmd = ClientCommandSelectCardToBuy::default();
        assert!(!cmd.initial_deck_choice);
        assert!(!cmd.first_card_choice);
    }

    #[test]
    fn both_true_stored() {
        let cmd = ClientCommandSelectCardToBuy::new(true, true);
        assert!(cmd.is_initial_deck_choice());
        assert!(cmd.is_first_card_choice());
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandSelectCardToBuy::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandSelectCardToBuy::default().clone();
    }

    #[test]
    fn get_id_is_client_select_card_to_buy() {
        assert_eq!(ClientCommandSelectCardToBuy::new(true, true).get_id(), NetCommandId::ClientSelectCardToBuy);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_card_selection() {
        let cmd = ClientCommandSelectCardToBuy::new(true, false);
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientSelectCardToBuy");
        assert_eq!(json["cardSelection"], "INITIAL_SECOND");
    }

    #[test]
    fn round_trip_populated() {
        let mut cmd = ClientCommandSelectCardToBuy::new(false, true);
        cmd.entropy = Some(9);
        let json = cmd.to_json_value();
        let restored = ClientCommandSelectCardToBuy::from_json(&json);
        assert_eq!(restored.initial_deck_choice, false);
        assert_eq!(restored.first_card_choice, true);
        assert_eq!(restored.entropy, Some(9));
    }

    #[test]
    fn round_trip_default() {
        let cmd = ClientCommandSelectCardToBuy::default();
        let json = cmd.to_json_value();
        let restored = ClientCommandSelectCardToBuy::from_json(&json);
        assert_eq!(restored.initial_deck_choice, cmd.initial_deck_choice);
        assert_eq!(restored.first_card_choice, cmd.first_card_choice);
        assert_eq!(restored.entropy, None);
    }
}
