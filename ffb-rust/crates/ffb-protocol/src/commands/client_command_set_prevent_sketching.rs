/// 1:1 translation of ClientCommandSetPreventSketching (Java fields: coach, preventSketching).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandSetPreventSketching {
    pub coach: Option<String>,
    pub prevent_sketching: bool,
}

impl ClientCommandSetPreventSketching {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_fields(coach: impl Into<String>, prevent_sketching: bool) -> Self {
        Self {
            coach: Some(coach.into()),
            prevent_sketching,
        }
    }

    pub fn get_coach(&self) -> Option<&str> {
        self.coach.as_deref()
    }

    pub fn is_prevent_sketching(&self) -> bool {
        self.prevent_sketching
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_no_coach_and_false_flag() {
        let cmd = ClientCommandSetPreventSketching::new();
        assert!(cmd.get_coach().is_none());
        assert!(!cmd.is_prevent_sketching());
    }

    #[test]
    fn with_fields_stores_values() {
        let cmd = ClientCommandSetPreventSketching::with_fields("coach-1", true);
        assert_eq!(cmd.get_coach(), Some("coach-1"));
        assert!(cmd.is_prevent_sketching());
    }
}
