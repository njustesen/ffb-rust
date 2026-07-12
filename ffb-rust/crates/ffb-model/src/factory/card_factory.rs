use crate::inducement::card::Card;
use crate::inducement::cards::Cards;

/// 1:1 translation of `com.fumbbl.ffb.factory.CardFactory`.
///
/// Java holds a `Cards` instance selected via `Scanner<>(Cards.class)` reflection over the
/// active game's rules/options at `initialize(Game)`; Rust has no runtime reflection, so the
/// concrete `Cards` implementation (e.g. `bb2016::cards::Cards`, `bb2020::cards::Cards`) is
/// passed in directly by the caller instead.
///
/// Java's `getShortName()`-based `forShortName(String)` lookup is intentionally omitted here:
/// the Rust `Card` model (`crate::inducement::card::Card`) does not carry a short-name field —
/// a pre-existing gap in the `Card`/`Cards` translation, out of scope for this factory.
pub struct CardFactory<'a> {
    cards: &'a dyn Cards,
}

impl<'a> CardFactory<'a> {
    pub fn new(cards: &'a dyn Cards) -> Self {
        Self { cards }
    }

    /// Java: `forName(String)` — case-insensitive full-name lookup.
    pub fn for_name(&self, name: &str) -> Option<&Card> {
        self.cards
            .all_cards()
            .iter()
            .find(|card| card.get_name().eq_ignore_ascii_case(name))
    }

    /// Java: `allCards()`.
    pub fn all_cards(&self) -> &[Card] {
        self.cards.all_cards()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inducement::bb2016::cards::Cards as Bb2016Cards;

    #[test]
    fn for_name_finds_card_case_insensitively() {
        let cards = Bb2016Cards::new();
        let factory = CardFactory::new(&cards);
        let card = factory.for_name("force shield").unwrap();
        assert_eq!(card.get_name(), "Force Shield");
    }

    #[test]
    fn for_name_returns_none_for_unknown() {
        let cards = Bb2016Cards::new();
        let factory = CardFactory::new(&cards);
        assert!(factory.for_name("Nonexistent Card").is_none());
    }

    #[test]
    fn all_cards_matches_underlying_collection() {
        let cards = Bb2016Cards::new();
        let factory = CardFactory::new(&cards);
        assert_eq!(factory.all_cards().len(), cards.all_cards().len());
    }

    #[test]
    fn for_name_finds_card_without_handler_key() {
        let cards = Bb2016Cards::new();
        let factory = CardFactory::new(&cards);
        let card = factory.for_name("Beguiling Bracers").unwrap();
        assert!(card.handler_key_name().is_none());
    }
}
