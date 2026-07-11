use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;
use ffb_model::enums::NetCommandId;

/// 1:1 translation of com.fumbbl.ffb.net.commands.ClientCommandBuyCard.
///
/// Java: `fCardType` is a `CardType` object (unit struct in Rust — identified by name).
/// We store the card type name as a String since CardType is a factory-created unit struct.

#[derive(Debug, Clone, Default)]
pub struct ClientCommandBuyCard {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `fCardType` — stored as its string identifier since CardType is a unit struct.
    pub card_type_name: Option<String>,
}

impl ClientCommandBuyCard {
    pub fn new() -> Self {
        Self::default()
    }

    /// Java: `getCardType()` — returns the card type name identifier.
    pub fn get_card_type_name(&self) -> Option<&str> {
        self.card_type_name.as_deref()
    }

    /// Java: `ClientCommandBuyCard.toJsonValue()` (calls `super.toJsonValue()` first).
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        if let Some(card_type) = &self.card_type_name {
            map.insert("cardType".to_string(), serde_json::json!(card_type));
        }
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandBuyCard.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            card_type_name: json.get("cardType").and_then(|v| v.as_str()).map(|s| s.to_string()),
        }
    }
}

impl NetCommand for ClientCommandBuyCard {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientBuyCard
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_no_card_type() {
        let cmd = ClientCommandBuyCard::new();
        assert!(cmd.get_card_type_name().is_none());
    }

    #[test]
    fn stores_card_type_name() {
        let cmd = ClientCommandBuyCard {
            entropy: None,
            card_type_name: Some("BRIBE".to_string()),
        };
        assert_eq!(cmd.get_card_type_name(), Some("BRIBE"));
    }

    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandBuyCard::new()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandBuyCard::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandBuyCard::default());
        assert!(s.contains("ClientCommandBuyCard"));
    }

    #[test]
    fn get_id_is_client_buy_card() {
        assert_eq!(ClientCommandBuyCard::new().get_id(), NetCommandId::ClientBuyCard);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_card_type() {
        let cmd = ClientCommandBuyCard {
            entropy: None,
            card_type_name: Some("BRIBE".to_string()),
        };
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientBuyCard");
        assert_eq!(json["cardType"], "BRIBE");
    }

    #[test]
    fn round_trip_with_card_type_and_entropy() {
        let cmd = ClientCommandBuyCard {
            entropy: Some(4),
            card_type_name: Some("BLOODWEISER_BABES".to_string()),
        };
        let json = cmd.to_json_value();
        let restored = ClientCommandBuyCard::from_json(&json);
        assert_eq!(restored.entropy, Some(4));
        assert_eq!(restored.get_card_type_name(), Some("BLOODWEISER_BABES"));
    }

    #[test]
    fn round_trip_with_no_card_type() {
        let cmd = ClientCommandBuyCard::new();
        let json = cmd.to_json_value();
        let restored = ClientCommandBuyCard::from_json(&json);
        assert!(restored.get_card_type_name().is_none());
        assert_eq!(restored.entropy, None);
    }
}
