/// 1:1 translation of ClientCommandPickUpChoice (Java field: attemptPickUp).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandPickUpChoice {
    pub attempt_pick_up: bool,
}

impl ClientCommandPickUpChoice {
    pub fn new(attempt_pick_up: bool) -> Self {
        Self { attempt_pick_up }
    }

    pub fn is_attempt_pick_up(&self) -> bool {
        self.attempt_pick_up
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_true_stores_true() {
        let cmd = ClientCommandPickUpChoice::new(true);
        assert!(cmd.is_attempt_pick_up());
    }

    #[test]
    fn new_false_stores_false() {
        let cmd = ClientCommandPickUpChoice::new(false);
        assert!(!cmd.is_attempt_pick_up());
    }

    #[test]
    fn default_is_false() {
        let cmd = ClientCommandPickUpChoice::default();
        assert!(!cmd.is_attempt_pick_up());
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandPickUpChoice::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandPickUpChoice::default().clone();
    }
}
