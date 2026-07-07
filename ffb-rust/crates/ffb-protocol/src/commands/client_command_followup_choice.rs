/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandFollowupChoice`.
/// Sent when the attacker decides whether to follow up after a block pushback.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandFollowupChoice {
    /// Java: `fChoiceFollowup`
    pub choice_followup: bool,
}

impl ClientCommandFollowupChoice {
    pub fn new(choice_followup: bool) -> Self {
        Self { choice_followup }
    }

    /// Java: `isChoiceFollowup()`
    pub fn is_choice_followup(&self) -> bool { self.choice_followup }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn followup_true_stored() {
        let cmd = ClientCommandFollowupChoice::new(true);
        assert!(cmd.is_choice_followup());
    }

    #[test]
    fn followup_false_stored() {
        let cmd = ClientCommandFollowupChoice::new(false);
        assert!(!cmd.is_choice_followup());
    }

    #[test]
    fn default_no_followup() {
        let cmd = ClientCommandFollowupChoice::default();
        assert!(!cmd.choice_followup);
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandFollowupChoice::default();
        assert!(!format!("{cmd:?}").is_empty());
    }
}
