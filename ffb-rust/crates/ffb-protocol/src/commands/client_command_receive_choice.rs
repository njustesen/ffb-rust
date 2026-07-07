/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandReceiveChoice`.
/// Sent when the coach chooses to receive or kick at the start of a half.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandReceiveChoice {
    /// Java: `fChoiceReceive`
    pub choice_receive: bool,
}

impl ClientCommandReceiveChoice {
    pub fn new(choice_receive: bool) -> Self {
        Self { choice_receive }
    }

    /// Java: `isChoiceReceive()`
    pub fn is_choice_receive(&self) -> bool { self.choice_receive }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn receive_true_stored() {
        let cmd = ClientCommandReceiveChoice::new(true);
        assert!(cmd.is_choice_receive());
    }

    #[test]
    fn kick_stored() {
        let cmd = ClientCommandReceiveChoice::new(false);
        assert!(!cmd.is_choice_receive());
    }

    #[test]
    fn default_is_kick() {
        let cmd = ClientCommandReceiveChoice::default();
        assert!(!cmd.choice_receive);
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandReceiveChoice::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandReceiveChoice::default().clone();
    }
}
