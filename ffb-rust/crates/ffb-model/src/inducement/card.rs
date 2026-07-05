/// 1:1 translation of `com.fumbbl.ffb.inducement.Card`.
/// Represents a single inducement card with its handler key and properties.
#[derive(Debug, Clone, Default)]
pub struct Card {
    pub name: String,
    /// Name of the CardHandlerKey variant that handles this card (matches Java handlerKey().name()).
    pub handler_key_name: Option<String>,
}

impl Card {
    pub fn new(name: impl Into<String>, handler_key_name: Option<impl Into<String>>) -> Self {
        Card {
            name: name.into(),
            handler_key_name: handler_key_name.map(|s| s.into()),
        }
    }

    /// Java: Card.getName()
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Java: Card.handlerKey() — returns the handler key name if one is set.
    pub fn handler_key_name(&self) -> Option<&str> {
        self.handler_key_name.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_with_handler_key_name() {
        let c = Card::new("Chop Block", Some("CHOP_BLOCK"));
        assert_eq!(c.get_name(), "Chop Block");
        assert_eq!(c.handler_key_name(), Some("CHOP_BLOCK"));
    }

    #[test]
    fn card_without_handler_key() {
        let c = Card::new("Bribe", None::<&str>);
        assert_eq!(c.get_name(), "Bribe");
        assert!(c.handler_key_name().is_none());
    }

    #[test]
    fn card_default_is_empty() {
        let c = Card::default();
        assert!(c.get_name().is_empty());
        assert!(c.handler_key_name().is_none());
    }
}
