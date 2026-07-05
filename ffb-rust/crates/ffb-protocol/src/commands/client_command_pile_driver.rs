/// 1:1 translation of ClientCommandPileDriver (Java field: playerId).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandPileDriver {
    pub player_id: Option<String>,
}

impl ClientCommandPileDriver {
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
        let cmd = ClientCommandPileDriver::new();
        assert!(cmd.get_player_id().is_none());
    }

    #[test]
    fn with_player_id_stores_value() {
        let cmd = ClientCommandPileDriver::with_player_id("p-7");
        assert_eq!(cmd.get_player_id(), Some("p-7"));
    }
}
