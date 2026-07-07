/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandRemovePlayer`.
/// Instructs the client to remove a player from the game view.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandRemovePlayer {
    /// Java: `fPlayerId` — the player to remove.
    pub player_id: String,
}

impl ServerCommandRemovePlayer {
    pub fn new(player_id: impl Into<String>) -> Self {
        Self { player_id: player_id.into() }
    }
    pub fn get_player_id(&self) -> &str { &self.player_id }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_id_stored() {
        let cmd = ServerCommandRemovePlayer::new("p1");
        assert_eq!(cmd.get_player_id(), "p1");
    }

    #[test]
    fn default_empty() {
        let cmd = ServerCommandRemovePlayer::default();
        assert!(cmd.player_id.is_empty());
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ServerCommandRemovePlayer::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ServerCommandRemovePlayer::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ServerCommandRemovePlayer::default());
        assert!(s.contains("ServerCommandRemovePlayer"));
    }
}
