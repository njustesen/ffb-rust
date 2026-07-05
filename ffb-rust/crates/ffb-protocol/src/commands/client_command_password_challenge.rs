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
}
