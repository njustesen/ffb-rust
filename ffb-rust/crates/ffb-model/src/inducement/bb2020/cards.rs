use crate::inducement::card::Card;
use crate::inducement::cards::Cards as ICards;

/// 1:1 translation of `com.fumbbl.ffb.inducement.bb2020.Cards`.
///
/// BB2020 card set — no cards defined in the Java source (empty set).
pub struct Cards {
    cards: Vec<Card>,
}

impl Default for Cards {
    fn default() -> Self {
        Self::new()
    }
}

impl Cards {
    pub fn new() -> Self {
        // Java: `private final Set<Card> cards = new HashSet<>()` — empty in BB2020
        Self { cards: vec![] }
    }
}

impl ICards for Cards {
    fn get_key(&self) -> &str {
        "Cards"
    }

    fn all_cards(&self) -> &[Card] {
        &self.cards
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inducement::cards::Cards as ICards;

    #[test]
    fn bb2020_cards_is_empty() {
        let c = Cards::new();
        assert_eq!(c.all_cards().len(), 0);
    }

    #[test]
    fn key_is_non_empty() {
        let c = Cards::new();
        assert!(!c.get_key().is_empty());
    }
}
