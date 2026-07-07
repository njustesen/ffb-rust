/// 1:1 translation of com.fumbbl.ffb.net.commands.ClientCommandKickTeamMate.

#[derive(Debug, Clone, Default)]
pub struct ClientCommandKickTeamMate {
    /// Java: `fKickedPlayerId`
    pub kicked_player_id: Option<String>,
    /// Java: `fActingPlayerId`
    pub acting_player_id: Option<String>,
    /// Java: `fNumDice`
    pub num_dice: i32,
}

impl ClientCommandKickTeamMate {
    pub fn new() -> Self {
        Self::default()
    }

    /// Java: `getKickedPlayerId()`
    pub fn get_kicked_player_id(&self) -> Option<&str> {
        self.kicked_player_id.as_deref()
    }

    /// Java: `getActingPlayerId()`
    pub fn get_acting_player_id(&self) -> Option<&str> {
        self.acting_player_id.as_deref()
    }

    /// Java: `getNumDice()`
    pub fn get_num_dice(&self) -> i32 {
        self.num_dice
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_num_dice_is_zero() {
        let cmd = ClientCommandKickTeamMate::new();
        assert_eq!(cmd.get_num_dice(), 0);
    }

    #[test]
    fn stores_player_ids_and_num_dice() {
        let cmd = ClientCommandKickTeamMate {
            kicked_player_id: Some("kicked_1".to_string()),
            acting_player_id: Some("acting_1".to_string()),
            num_dice: 2,
        };
        assert_eq!(cmd.get_kicked_player_id(), Some("kicked_1"));
        assert_eq!(cmd.get_acting_player_id(), Some("acting_1"));
        assert_eq!(cmd.get_num_dice(), 2);
    }

    #[test]
    fn default_ids_are_none() {
        let cmd = ClientCommandKickTeamMate::default();
        assert!(cmd.get_kicked_player_id().is_none());
        assert!(cmd.get_acting_player_id().is_none());
    }


    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandKickTeamMate::default();
        assert!(!format!("{cmd:?}").is_empty());
    }
}
