/// 1:1 translation of ClientCommandThrowKeg (Java field: playerId).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandThrowKeg {
    pub player_id: Option<String>,
}

impl ClientCommandThrowKeg {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_player_id(id: impl Into<String>) -> Self {
        Self { player_id: Some(id.into()) }
    }

    pub fn get_player_id(&self) -> Option<&str> {
        self.player_id.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_no_player_id() {
        let cmd = ClientCommandThrowKeg::new();
        assert!(cmd.get_player_id().is_none());
    }

    #[test]
    fn with_player_id_stores_value() {
        let cmd = ClientCommandThrowKeg::with_player_id("p-42");
        assert_eq!(cmd.get_player_id(), Some("p-42"));
    }
}
