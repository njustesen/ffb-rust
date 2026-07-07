/// 1:1 translation of com.fumbbl.ffb.net.commands.ClientCommandJoinReplay.

#[derive(Debug, Clone, Default)]
pub struct ClientCommandJoinReplay {
    /// Java: `replayName`
    pub replay_name: Option<String>,
    /// Java: `coach`
    pub coach: Option<String>,
    /// Java: `gameId`
    pub game_id: i64,
}

impl ClientCommandJoinReplay {
    pub fn new() -> Self {
        Self::default()
    }

    /// Java: `getReplayName()`
    pub fn get_replay_name(&self) -> Option<&str> {
        self.replay_name.as_deref()
    }

    /// Java: `getCoach()`
    pub fn get_coach(&self) -> Option<&str> {
        self.coach.as_deref()
    }

    /// Java: `getGameId()`
    pub fn get_game_id(&self) -> i64 {
        self.game_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_game_id_is_zero() {
        let cmd = ClientCommandJoinReplay::new();
        assert_eq!(cmd.get_game_id(), 0);
    }

    #[test]
    fn stores_replay_name_and_coach() {
        let cmd = ClientCommandJoinReplay {
            replay_name: Some("replay_001".to_string()),
            coach: Some("CoachA".to_string()),
            game_id: 99,
        };
        assert_eq!(cmd.get_replay_name(), Some("replay_001"));
        assert_eq!(cmd.get_coach(), Some("CoachA"));
        assert_eq!(cmd.get_game_id(), 99);
    }

    #[test]
    fn replay_name_none_by_default() {
        let cmd = ClientCommandJoinReplay::default();
        assert!(cmd.get_replay_name().is_none());
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandJoinReplay::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandJoinReplay::default().clone();
    }
}
