/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandZapPlayer`.
/// Instructs the client to visually remove (zap) a player from the field.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandZapPlayer {
    /// Java: `teamId` — the team the player belongs to.
    pub team_id: String,
    /// Java: `playerId` — the player to zap.
    pub player_id: String,
}

impl ServerCommandZapPlayer {
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
        let cmd = ServerCommandZapPlayer::new("team2", "p5");
        assert_eq!(cmd.get_team_id(), "team2");
        assert_eq!(cmd.get_player_id(), "p5");
    }

    #[test]
    fn default_empty() {
        let cmd = ServerCommandZapPlayer::default();
        assert!(cmd.player_id.is_empty());
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ServerCommandZapPlayer::default()).is_empty());
    }

}
