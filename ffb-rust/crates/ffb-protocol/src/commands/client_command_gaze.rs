/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandGaze`.
/// Sent when a player uses Hypnotic Gaze on a victim.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandGaze {
    /// Java: `fActingPlayerId`
    pub acting_player_id: Option<String>,
    /// Java: `fVictimId`
    pub victim_id: Option<String>,
}

impl ClientCommandGaze {
    pub fn new() -> Self { Self::default() }

    pub fn with_players(
        acting_player_id: impl Into<String>,
        victim_id: impl Into<String>,
    ) -> Self {
        Self {
            acting_player_id: Some(acting_player_id.into()),
            victim_id: Some(victim_id.into()),
        }
    }

    pub fn get_acting_player_id(&self) -> Option<&str> { self.acting_player_id.as_deref() }
    pub fn get_victim_id(&self) -> Option<&str> { self.victim_id.as_deref() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored_correctly() {
        let cmd = ClientCommandGaze::with_players("gazer", "victim");
        assert_eq!(cmd.get_acting_player_id(), Some("gazer"));
        assert_eq!(cmd.get_victim_id(), Some("victim"));
    }

    #[test]
    fn default_both_none() {
        let cmd = ClientCommandGaze::new();
        assert!(cmd.acting_player_id.is_none());
        assert!(cmd.victim_id.is_none());
    }
#[test]    fn new_constructor_creates_default() { let cmd = ClientCommandGaze::new(); let _ = cmd; }

    #[test]
    fn victim_id_stored() {
        let cmd = ClientCommandGaze::with_players("g", "v");
        assert_eq!(cmd.get_victim_id(), Some("v"));
    }
}
