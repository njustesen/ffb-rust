use ffb_model::model::client_mode::ClientMode;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandJoin`.
/// Notifies clients when a coach joins the game.
#[derive(Debug, Clone)]
pub struct ServerCommandJoin {
    /// Java: `fCoach` — coach name.
    pub coach: String,
    /// Java: `fClientMode` — connection mode (Home/Away/Spectator/Replay).
    pub client_mode: ClientMode,
    /// Java: `fPlayerNames` — list of logged-in player names.
    pub player_names: Vec<String>,
    /// Java: `spectators` — list of spectator coach names.
    pub spectators: Vec<String>,
    /// Java: `replayName` — name of replay being watched (if any).
    pub replay_name: String,
}

impl ServerCommandJoin {
    pub fn new(
        coach: impl Into<String>,
        client_mode: ClientMode,
        player_names: Vec<String>,
        spectators: Vec<String>,
        replay_name: impl Into<String>,
    ) -> Self {
        Self {
            coach: coach.into(),
            client_mode,
            player_names,
            spectators,
            replay_name: replay_name.into(),
        }
    }
    pub fn get_coach(&self) -> &str { &self.coach }
    pub fn get_client_mode(&self) -> ClientMode { self.client_mode }
    pub fn get_player_names(&self) -> &[String] { &self.player_names }
    pub fn get_spectators(&self) -> &[String] { &self.spectators }
    pub fn get_spectator_count(&self) -> usize { self.spectators.len() }
    pub fn get_replay_name(&self) -> &str { &self.replay_name }
}

impl Default for ServerCommandJoin {
    fn default() -> Self {
        Self {
            coach: String::new(),
            client_mode: ClientMode::PLAYER,
            player_names: Vec::new(),
            spectators: Vec::new(),
            replay_name: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let cmd = ServerCommandJoin::new(
            "Alice",
            ClientMode::PLAYER,
            vec!["Alice".into()],
            vec!["Bob".into()],
            "",
        );
        assert_eq!(cmd.get_coach(), "Alice");
        assert_eq!(cmd.get_client_mode(), ClientMode::PLAYER);
        assert_eq!(cmd.get_spectator_count(), 1);
    }

    #[test]
    fn default_empty() {
        let cmd = ServerCommandJoin::default();
        assert!(cmd.coach.is_empty());
        assert!(cmd.player_names.is_empty());
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ServerCommandJoin::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_roundtrip() {
        let cmd = ServerCommandJoin::default();
        let _ = cmd.clone();
    }
}
