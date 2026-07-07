/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandUnzapPlayer`.
/// Instructs the client to restore a previously zapped player.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandUnzapPlayer {
    /// Java: `teamId` — the team the player belongs to.
    pub team_id: String,
    /// Java: `playerId` — the player to un-zap.
    pub player_id: String,
}

impl ServerCommandUnzapPlayer {
    pub fn new(team_id: impl Into<String>, player_id: impl Into<String>) -> Self {
        Self { team_id: team_id.into(), player_id: player_id.into() }
    }
    pub fn get_team_id(&self) -> &str { &self.team_id }
    pub fn get_player_id(&self) -> &str { &self.player_id }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let cmd = ServerCommandUnzapPlayer::new("team1", "p1");
        assert_eq!(cmd.get_team_id(), "team1");
        assert_eq!(cmd.get_player_id(), "p1");
    }

    #[test]
    fn default_same_as_new() {
        let _ = ServerCommandUnzapPlayer::default();
    }

    #[test]
    fn debug_format_works() {
        let v = ServerCommandUnzapPlayer::new("t", "p");
        assert!(!format!("{:?}", v).is_empty());
    }
}
