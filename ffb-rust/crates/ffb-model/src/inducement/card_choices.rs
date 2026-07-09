use crate::inducement::card_choice::CardChoice;

/// Pair of initial and rerolled card choices — 1:1 translation of Java CardChoices.
pub struct CardChoices {
    initial: Option<CardChoice>,
    rerolled: Option<CardChoice>,
}

impl CardChoices {
    pub fn new() -> Self {
        Self { initial: None, rerolled: None }
    }

    pub fn with_both(initial: CardChoice, rerolled: CardChoice) -> Self {
        Self { initial: Some(initial), rerolled: Some(rerolled) }
    }

    pub fn get_initial(&self) -> Option<&CardChoice> {
        self.initial.as_ref()
    }

    pub fn get_rerolled(&self) -> Option<&CardChoice> {
        self.rerolled.as_ref()
    }

    pub fn set_initial(&mut self, choice: CardChoice) {
        self.initial = Some(choice);
    }

    pub fn set_rerolled(&mut self, choice: CardChoice) {
        self.rerolled = Some(choice);
    }
}

impl Default for CardChoices {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_choices() {
        let choices = CardChoices::new();
        assert!(choices.get_initial().is_none());
        assert!(choices.get_rerolled().is_none());
    }

    #[test]
    fn test_set_choices() {
        let mut choices = CardChoices::new();
        choices.set_initial(CardChoice::new().with_type("CHAOS"));
        assert!(choices.get_initial().is_some());
        assert!(choices.get_rerolled().is_none());
    }
}
