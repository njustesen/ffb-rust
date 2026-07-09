/// 1:1 translation of `com.fumbbl.ffb.inducement.CardType` (interface).
///
/// Describes a category of cards (e.g. Magic Items, Dirty Tricks).
pub trait CardType {
    /// Java: `getName()` — the internal name key (e.g. "magicItem").
    fn get_name(&self) -> &str;

    /// Java: `getDeckName()` — human-readable deck name (e.g. "Magic Items Deck").
    fn get_deck_name(&self) -> &str;

    /// Java: `getInducementNameSingle()`
    fn get_inducement_name_single(&self) -> &str;

    /// Java: `getInducementNameMultiple()`
    fn get_inducement_name_multiple(&self) -> &str;

    /// Java: `getMaxId()` — game option key for maximum cards allowed.
    fn get_max_id(&self) -> &str;

    /// Java: `getCostId()` — game option key for cost per card.
    fn get_cost_id(&self) -> &str;

    /// Java: `getCardFront()` — icon/animation property for card front.
    fn get_card_front(&self) -> &str;

    /// Java: `getCardBack()` — icon/animation property for card back.
    fn get_card_back(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestCardType;
    impl CardType for TestCardType {
        fn get_name(&self) -> &str { "magicItem" }
        fn get_deck_name(&self) -> &str { "Magic Items Deck" }
        fn get_inducement_name_single(&self) -> &str { "Magic Item Card" }
        fn get_inducement_name_multiple(&self) -> &str { "Magic Item Cards" }
        fn get_max_id(&self) -> &str { "cardsMagicItemMax" }
        fn get_cost_id(&self) -> &str { "cardsMagicItemCost" }
        fn get_card_front(&self) -> &str { "cardMagicItemFront" }
        fn get_card_back(&self) -> &str { "cardMagicItemBack" }
    }

    #[test]
    fn get_name_returns_correct_key() {
        let ct = TestCardType;
        assert_eq!(ct.get_name(), "magicItem");
    }

    #[test]
    fn get_deck_name_is_non_empty() {
        let ct = TestCardType;
        assert!(!ct.get_deck_name().is_empty());
    }
}
