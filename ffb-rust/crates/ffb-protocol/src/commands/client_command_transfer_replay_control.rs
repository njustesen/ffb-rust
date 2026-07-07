/// 1:1 translation of ClientCommandTransferReplayControl (Java field: coach).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandTransferReplayControl {
    pub coach: Option<String>,
}

impl ClientCommandTransferReplayControl {
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
        let cmd = ClientCommandTransferReplayControl::new();
        assert!(cmd.get_coach().is_none());
    }

    #[test]
    fn with_coach_stores_value() {
        let cmd = ClientCommandTransferReplayControl::with_coach("coach-abc");
        assert_eq!(cmd.get_coach(), Some("coach-abc"));
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandTransferReplayControl::default()).is_empty());
    }

}
