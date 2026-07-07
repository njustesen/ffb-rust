/// 1:1 translation of ClientCommandTargetSelected (Java field: targetPlayerId).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandTargetSelected {
    pub target_player_id: Option<String>,
}

impl ClientCommandTargetSelected {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_target(id: impl Into<String>) -> Self {
        Self { target_player_id: Some(id.into()) }
    }

    pub fn get_target_player_id(&self) -> Option<&str> {
        self.target_player_id.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_no_target() {
        let cmd = ClientCommandTargetSelected::new();
        assert!(cmd.get_target_player_id().is_none());
    }

    #[test]
    fn with_target_stores_value() {
        let cmd = ClientCommandTargetSelected::with_target("player-123");
        assert_eq!(cmd.get_target_player_id(), Some("player-123"));
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandTargetSelected::default()).is_empty());
    }

}
