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
