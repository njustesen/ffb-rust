/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandBloodlustAction`.
/// Sent when a vampire player chooses to change their blood lust action.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandBloodlustAction {
    /// Java: `change` — whether the player wants to change their action.
    pub change: bool,
}

impl ClientCommandBloodlustAction {
    pub fn new(change: bool) -> Self { Self { change } }
    pub fn is_change(&self) -> bool { self.change }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn change_true_stored() {
        let cmd = ClientCommandBloodlustAction::new(true);
        assert!(cmd.is_change());
    }

    #[test]
    fn default_is_false() {
        let cmd = ClientCommandBloodlustAction::default();
        assert!(!cmd.change);
    }

    #[test]
    fn change_false_stored() {
        let cmd = ClientCommandBloodlustAction::new(false);
        assert!(!cmd.is_change());
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandBloodlustAction::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandBloodlustAction::default().clone();
    }
}
