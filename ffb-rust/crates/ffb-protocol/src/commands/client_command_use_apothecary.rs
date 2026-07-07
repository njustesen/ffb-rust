/// 1:1 translation of com.fumbbl.ffb.net.commands.ClientCommandUseApothecary.
use ffb_model::enums::{ApothecaryType, PlayerState, SeriousInjuryKind};

#[derive(Debug, Clone, Default)]
pub struct ClientCommandUseApothecary {
    /// Java: `fPlayerId`
    pub player_id: Option<String>,
    /// Java: `fApothecaryUsed`
    pub apothecary_used: bool,
    /// Java: `apothecaryType`
    pub apothecary_type: Option<ApothecaryType>,
    /// Java: `seriousInjury` — simplified to SeriousInjuryKind (the injury kind field).
    pub serious_injury: Option<SeriousInjuryKind>,
    /// Java: `playerState`
    pub player_state: Option<PlayerState>,
}

impl ClientCommandUseApothecary {
    pub fn new() -> Self {
        Self::default()
    }

    /// Java: `getPlayerId()`
    pub fn get_player_id(&self) -> Option<&str> {
        self.player_id.as_deref()
    }

    /// Java: `isApothecaryUsed()`
    pub fn is_apothecary_used(&self) -> bool {
        self.apothecary_used
    }

    /// Java: `getApothecaryType()`
    pub fn get_apothecary_type(&self) -> Option<ApothecaryType> {
        self.apothecary_type
    }

    /// Java: `getSeriousInjury()`
    pub fn get_serious_injury(&self) -> Option<SeriousInjuryKind> {
        self.serious_injury
    }

    /// Java: `getPlayerState()`
    pub fn get_player_state(&self) -> Option<PlayerState> {
        self.player_state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_apothecary_not_used() {
        let cmd = ClientCommandUseApothecary::new();
        assert!(!cmd.is_apothecary_used());
        assert!(cmd.get_player_id().is_none());
    }

    #[test]
    fn stores_apothecary_fields() {
        let cmd = ClientCommandUseApothecary {
            player_id: Some("player_3".to_string()),
            apothecary_used: true,
            apothecary_type: Some(ApothecaryType::Team),
            serious_injury: Some(SeriousInjuryKind::SeriouslyHurt),
            player_state: Some(PlayerState::new(1)),
        };
        assert_eq!(cmd.get_player_id(), Some("player_3"));
        assert!(cmd.is_apothecary_used());
        assert_eq!(cmd.get_apothecary_type(), Some(ApothecaryType::Team));
        assert_eq!(cmd.get_serious_injury(), Some(SeriousInjuryKind::SeriouslyHurt));
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandUseApothecary::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandUseApothecary::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandUseApothecary::default());
        assert!(s.contains("ClientCommandUseApothecary"));
    }
}
