use ffb_model::model::client_mode::ClientMode;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandLeave`.
/// Notifies clients when a coach leaves the game.
#[derive(Debug, Clone)]
pub struct ServerCommandLeave {
    /// Java: `fCoach` — leaving coach name.
    pub coach: String,
    /// Java: `fClientMode` — connection mode of the leaving coach.
    pub client_mode: ClientMode,
    /// Java: `spectators` — updated list of spectator coach names.
    pub spectators: Vec<String>,
}

impl ServerCommandLeave {
    pub fn new(
        coach: impl Into<String>,
        client_mode: ClientMode,
        spectators: Vec<String>,
    ) -> Self {
        Self { coach: coach.into(), client_mode, spectators }
    }
    pub fn get_coach(&self) -> &str { &self.coach }
    pub fn get_client_mode(&self) -> ClientMode { self.client_mode }
    pub fn get_spectators(&self) -> &[String] { &self.spectators }
    pub fn get_spectator_count(&self) -> usize { self.spectators.len() }
}

impl Default for ServerCommandLeave {
    fn default() -> Self {
        Self {
            coach: String::new(),
            client_mode: ClientMode::PLAYER,
            spectators: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let cmd = ServerCommandLeave::new("Bob", ClientMode::SPECTATOR, vec!["Charlie".into()]);
        assert_eq!(cmd.get_coach(), "Bob");
        assert_eq!(cmd.get_client_mode(), ClientMode::SPECTATOR);
        assert_eq!(cmd.get_spectator_count(), 1);
    }

    #[test]
    fn default_empty() {
        let cmd = ServerCommandLeave::default();
        assert!(cmd.coach.is_empty());
    }
}
