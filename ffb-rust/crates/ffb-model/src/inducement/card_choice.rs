/// A pair of cards to choose between — 1:1 translation of Java CardChoice.
pub struct CardChoice {
    card_type: Option<String>,
    choice_one: Option<String>,
    choice_two: Option<String>,
}

impl CardChoice {
    pub fn new() -> Self {
        Self { card_type: None, choice_one: None, choice_two: None }
    }

    pub fn with_type(mut self, card_type: impl Into<String>) -> Self {
        self.card_type = Some(card_type.into());
        self
    }

    pub fn with_choice_one(mut self, card: impl Into<String>) -> Self {
        self.choice_one = Some(card.into());
        self
    }

    pub fn with_choice_two(mut self, card: impl Into<String>) -> Self {
        self.choice_two = Some(card.into());
        self
    }

    pub fn get_type(&self) -> Option<&str> {
        self.card_type.as_deref()
    }

    pub fn get_choice_one(&self) -> Option<&str> {
        self.choice_one.as_deref()
    }

    pub fn get_choice_two(&self) -> Option<&str> {
        self.choice_two.as_deref()
    }
}

impl Default for CardChoice {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_pattern() {
        let choice = CardChoice::new()
            .with_type("CHAOS")
            .with_choice_one("FireBall")
            .with_choice_two("IceBolt");
        assert_eq!(choice.get_type(), Some("CHAOS"));
        assert_eq!(choice.get_choice_one(), Some("FireBall"));
    }

    #[test]
    fn test_empty_choice() {
        let choice = CardChoice::new();
        assert!(choice.get_type().is_none());
        assert!(choice.get_choice_one().is_none());
    }
}
