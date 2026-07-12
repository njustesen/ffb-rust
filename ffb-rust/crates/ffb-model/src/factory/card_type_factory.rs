use crate::inducement::card_type::CardType;

/// 1:1 translation of `com.fumbbl.ffb.factory.CardTypeFactory`.
///
/// Java populates its `Set<CardType>` via `Scanner<>(CardType.class).getEnumValues(options)`
/// reflection at `initialize(Game)`; Rust has no runtime reflection, so the concrete `CardType`
/// enum values (e.g. `bb2016::card_type::CardType`, `bb2020::card_type::CardType`) are passed in
/// directly by the caller instead.
pub struct CardTypeFactory<'a> {
    card_types: Vec<&'a dyn CardType>,
}

impl<'a> CardTypeFactory<'a> {
    pub fn new(card_types: Vec<&'a dyn CardType>) -> Self {
        Self { card_types }
    }

    /// Java: `forName(String)` — case-insensitive lookup.
    pub fn for_name(&self, name: &str) -> Option<&'a dyn CardType> {
        self.card_types
            .iter()
            .copied()
            .find(|card_type| card_type.get_name().eq_ignore_ascii_case(name))
    }

    /// Java: `getCardTypes()`.
    pub fn get_card_types(&self) -> &[&'a dyn CardType] {
        &self.card_types
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inducement::bb2016::card_type::CardType as Bb2016CardType;

    #[test]
    fn for_name_finds_magic_item() {
        let magic = Bb2016CardType::MAGIC_ITEM;
        let dirty = Bb2016CardType::DIRTY_TRICK;
        let factory = CardTypeFactory::new(vec![&magic, &dirty]);
        let found = factory.for_name("magicItem").unwrap();
        assert_eq!(found.get_name(), "magicItem");
    }

    #[test]
    fn for_name_is_case_insensitive() {
        let magic = Bb2016CardType::MAGIC_ITEM;
        let factory = CardTypeFactory::new(vec![&magic]);
        assert!(factory.for_name("MAGICITEM").is_some());
    }

    #[test]
    fn for_name_returns_none_for_unknown() {
        let magic = Bb2016CardType::MAGIC_ITEM;
        let factory = CardTypeFactory::new(vec![&magic]);
        assert!(factory.for_name("unknown").is_none());
    }

    #[test]
    fn get_card_types_returns_all() {
        let magic = Bb2016CardType::MAGIC_ITEM;
        let dirty = Bb2016CardType::DIRTY_TRICK;
        let factory = CardTypeFactory::new(vec![&magic, &dirty]);
        assert_eq!(factory.get_card_types().len(), 2);
    }
}
