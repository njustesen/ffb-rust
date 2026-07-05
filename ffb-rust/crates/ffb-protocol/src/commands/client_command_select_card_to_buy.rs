/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandSelectCardToBuy`.
/// Java has an inner `Selection` class with `initialDeckChoice` and `firstCardChoice` booleans;
/// simplified here to two top-level fields.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandSelectCardToBuy {
    /// Java: `selection.initialDeckChoice`
    pub initial_deck_choice: bool,
    /// Java: `selection.firstCardChoice`
    pub first_card_choice: bool,
}

impl ClientCommandSelectCardToBuy {
    pub fn new(initial_deck_choice: bool, first_card_choice: bool) -> Self {
        Self { initial_deck_choice, first_card_choice }
    }

    pub fn is_initial_deck_choice(&self) -> bool { self.initial_deck_choice }
    pub fn is_first_card_choice(&self) -> bool { self.first_card_choice }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bools_stored() {
        let cmd = ClientCommandSelectCardToBuy::new(true, false);
        assert!(cmd.is_initial_deck_choice());
        assert!(!cmd.is_first_card_choice());
    }

    #[test]
    fn default_is_false() {
        let cmd = ClientCommandSelectCardToBuy::default();
        assert!(!cmd.initial_deck_choice);
        assert!(!cmd.first_card_choice);
    }
}
