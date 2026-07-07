use ffb_model::model::BlockKind;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandSetBlockTargetSelection`.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandSetBlockTargetSelection {
    /// Java: `playerId`
    pub player_id: Option<String>,
    /// Java: `kind`
    pub kind: Option<BlockKind>,
}

impl ClientCommandSetBlockTargetSelection {
    pub fn new() -> Self { Self::default() }

    pub fn with_target(player_id: impl Into<String>, kind: BlockKind) -> Self {
        Self { player_id: Some(player_id.into()), kind: Some(kind) }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_kind(&self) -> Option<BlockKind> { self.kind }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let cmd = ClientCommandSetBlockTargetSelection::with_target("p1", BlockKind::BLOCK);
        assert_eq!(cmd.get_player_id(), Some("p1"));
        assert_eq!(cmd.get_kind(), Some(BlockKind::BLOCK));
    }

    #[test]
    fn default_is_empty() {
        let cmd = ClientCommandSetBlockTargetSelection::new();
        assert!(cmd.player_id.is_none());
        assert!(cmd.kind.is_none());
    }

    #[test]
    fn stab_kind_stored() {
        let cmd = ClientCommandSetBlockTargetSelection::with_target("p2", BlockKind::STAB);
        assert_eq!(cmd.get_kind(), Some(BlockKind::STAB));
        assert_eq!(cmd.get_player_id(), Some("p2"));
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandSetBlockTargetSelection::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandSetBlockTargetSelection::default().clone();
    }
}
