/// 1:1 translation of ClientCommandPasswordChallenge (Java field: fCoach).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandPasswordChallenge {
    pub coach: Option<String>,
}

impl ClientCommandPasswordChallenge {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_coach(coach: impl Into<String>) -> Self {
        Self { coach: Some(coach.into()) }
    }

    pub fn get_coach(&self) -> Option<&str> {
        self.coach.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_no_coach() {
        let cmd = ClientCommandPasswordChallenge::new();
        assert!(cmd.get_coach().is_none());
    }

    #[test]
    fn with_coach_stores_value() {
        let cmd = ClientCommandPasswordChallenge::with_coach("coach-xyz");
        assert_eq!(cmd.get_coach(), Some("coach-xyz"));
    }

    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandPasswordChallenge::new()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandPasswordChallenge::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandPasswordChallenge::default());
        assert!(s.contains("ClientCommandPasswordChallenge"));
    }
}
