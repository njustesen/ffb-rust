/// 1:1 translation of com.fumbbl.ffb.net.commands.ClientCommandBuyCard.
///
/// Java: `fCardType` is a `CardType` object (unit struct in Rust — identified by name).
/// We store the card type name as a String since CardType is a factory-created unit struct.

#[derive(Debug, Clone, Default)]
pub struct ClientCommandBuyCard {
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
}
