use ffb_model::enums::{PlayerState, SeriousInjuryKind};

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandApothecaryChoice`.
/// Sent when a coach makes an apothecary decision for an injured player.
#[derive(Debug, Clone)]
pub struct ClientCommandApothecaryChoice {
    /// Java: `fPlayerId`
    pub player_id: Option<String>,
    /// Java: `fPlayerState`
    pub player_state: Option<PlayerState>,
    /// Java: `oldPlayerState`
    pub old_player_state: Option<PlayerState>,
    /// Java: `fSeriousInjury`
    pub serious_injury: Option<SeriousInjuryKind>,
}

impl Default for ClientCommandApothecaryChoice {
    fn default() -> Self {
        Self {
            player_id: None,
            player_state: None,
            old_player_state: None,
            serious_injury: None,
        }
    }
}

impl ClientCommandApothecaryChoice {
    pub fn new() -> Self { Self::default() }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_player_state(&self) -> Option<PlayerState> { self.player_state }
    pub fn get_old_player_state(&self) -> Option<PlayerState> { self.old_player_state }
    pub fn get_serious_injury(&self) -> Option<SeriousInjuryKind> { self.serious_injury }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_all_none() {
        let cmd = ClientCommandApothecaryChoice::new();
        assert!(cmd.player_id.is_none());
        assert!(cmd.player_state.is_none());
        assert!(cmd.old_player_state.is_none());
        assert!(cmd.serious_injury.is_none());
    }

    #[test]
    fn fields_accessible() {
        let mut cmd = ClientCommandApothecaryChoice::new();
        cmd.player_id = Some("p1".into());
        cmd.serious_injury = Some(SeriousInjuryKind::Dead);
        assert_eq!(cmd.get_player_id(), Some("p1"));
        assert_eq!(cmd.get_serious_injury(), Some(SeriousInjuryKind::Dead));
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandApothecaryChoice::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_roundtrip() {
        let cmd = ClientCommandApothecaryChoice::default();
        let _ = cmd.clone();
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandApothecaryChoice::default().clone();
    }
}
