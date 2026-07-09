use crate::inducement::card::Card;

/// 1:1 translation of `com.fumbbl.ffb.inducement.Cards` (interface).
///
/// A collection of all available cards for an edition.
pub trait Cards {
    /// Java: `getKey()` — returns class simple name as registry key.
    fn get_key(&self) -> &str;

    /// Java: `allCards()` — the full set of cards in this collection.
    fn all_cards(&self) -> &[Card];
}

#[cfg(test)]
mod tests {
    use super::*;

    struct EmptyCards;
    impl Cards for EmptyCards {
        fn get_key(&self) -> &str { "EmptyCards" }
        fn all_cards(&self) -> &[Card] { &[] }
    }

    #[test]
    fn empty_cards_key_is_non_empty() {
        let c = EmptyCards;
        assert!(!c.get_key().is_empty());
    }

    #[test]
    fn empty_cards_has_no_cards() {
        let c = EmptyCards;
        assert_eq!(c.all_cards().len(), 0);
    }
}
