/// Shuffled card deck for inducements — 1:1 translation of Java CardDeck.
pub struct CardDeck {
    card_type: String,
    cards: Vec<String>,
}

impl CardDeck {
    pub fn new(card_type: impl Into<String>) -> Self {
        Self { card_type: card_type.into(), cards: Vec::new() }
    }

    pub fn get_type(&self) -> &str {
        &self.card_type
    }

    pub fn add(&mut self, card: String) {
        self.cards.push(card);
    }

    pub fn remove(&mut self, card: &str) -> bool {
        if let Some(pos) = self.cards.iter().position(|c| c == card) {
            self.cards.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn draw(&mut self, index: usize) -> Option<String> {
        if index < self.cards.len() {
            Some(self.cards.remove(index))
        } else {
            None
        }
    }

    pub fn size(&self) -> usize {
        self.cards.len()
    }

    pub fn get_cards(&self) -> &[String] {
        &self.cards
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_deck_is_empty() {
        let deck = CardDeck::new("CHAOS");
        assert_eq!(deck.size(), 0);
        assert_eq!(deck.get_type(), "CHAOS");
    }

    #[test]
    fn test_add_and_draw() {
        let mut deck = CardDeck::new("CHAOS");
        deck.add("FireBall".to_string());
        deck.add("IceBolt".to_string());
        assert_eq!(deck.size(), 2);
        let drawn = deck.draw(0);
        assert_eq!(drawn, Some("FireBall".to_string()));
        assert_eq!(deck.size(), 1);
    }
}
