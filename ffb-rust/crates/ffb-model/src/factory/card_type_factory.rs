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

/// Java: `game.getFactory(Factory.CARD_TYPE).forName(name)` for a game of `rules` --
/// `initialize(Game)` fills the set with `Scanner<>(CardType.class).getEnumValues(options)`, i.e.
/// the values of the `CardType` enums whose `@RulesCollection` the ruleset is-or-extends:
/// `bb2016.CardType` for BB2016, `bb2020.CardType` for BB2020 and NONE for BB2025 (no BB2025
/// card type exists, so every name is null there). `forName` is `equalsIgnoreCase` and yields
/// the type, whose `getName()` is the canonical spelling. `None` = Java null.
pub fn for_name_in(rules: crate::enums::Rules, name: &str) -> Option<&'static str> {
    use crate::enums::Rules;
    use crate::inducement::bb2016::card_type::CardType as Bb2016CardType;
    use crate::inducement::bb2020::card_type::CardType as Bb2020CardType;
    static BB2016: [Bb2016CardType; 2] = [Bb2016CardType::MAGIC_ITEM, Bb2016CardType::DIRTY_TRICK];
    static BB2020: [Bb2020CardType; 2] = [Bb2020CardType::MAGIC_ITEM, Bb2020CardType::DIRTY_TRICK];
    let types: Vec<&'static dyn CardType> = match rules {
        Rules::Bb2016 => BB2016.iter().map(|t| t as &'static dyn CardType).collect(),
        Rules::Bb2020 => BB2020.iter().map(|t| t as &'static dyn CardType).collect(),
        Rules::Bb2025 | Rules::Common => Vec::new(),
    };
    types
        .into_iter()
        .map(|t| t.get_name())
        .find(|type_name| crate::util::java_json::java_equals_ignore_case(type_name, name))
}

#[cfg(test)]
mod tests {
    #[test]
    fn for_name_in_is_per_ruleset_and_returns_the_canonical_name() {
        use crate::enums::Rules;
        assert_eq!(super::for_name_in(Rules::Bb2016, "MAGICITEM"), Some("magicItem"));
        assert_eq!(super::for_name_in(Rules::Bb2020, "dirtyTrick"), Some("dirtyTrick"));
        // no BB2025 CardType enum: the factory's set is empty
        assert_eq!(super::for_name_in(Rules::Bb2025, "magicItem"), None);
        assert_eq!(super::for_name_in(Rules::Bb2016, "unknown"), None);
    }

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
