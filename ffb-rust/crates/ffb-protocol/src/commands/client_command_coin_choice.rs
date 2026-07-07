/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandCoinChoice`.
/// Sent by a client when choosing heads or tails for the coin flip.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandCoinChoice {
    /// Java: `fChoiceHeads`
    pub choice_heads: bool,
}

impl ClientCommandCoinChoice {
    pub fn new(choice_heads: bool) -> Self {
        Self { choice_heads }
    }

    /// Java: `isChoiceHeads()`
    pub fn is_choice_heads(&self) -> bool {
        self.choice_heads
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn choice_heads_stored() {
        let cmd = ClientCommandCoinChoice::new(true);
        assert!(cmd.is_choice_heads());
    }

    #[test]
    fn choice_tails_stored() {
        let cmd = ClientCommandCoinChoice::new(false);
        assert!(!cmd.is_choice_heads());
    }

    #[test]
    fn default_is_tails() {
        let cmd = ClientCommandCoinChoice::default();
        assert!(!cmd.choice_heads);
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandCoinChoice::default();
        assert!(!format!("{cmd:?}").is_empty());
    }
}
