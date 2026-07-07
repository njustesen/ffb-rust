/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandReplay`.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandReplay {
    /// Java: `fGameId`
    pub game_id: i64,
    /// Java: `fReplayToCommandNr`
    pub replay_to_command_nr: i32,
    /// Java: `coach`
    pub coach: Option<String>,
}

impl ClientCommandReplay {
    pub fn new() -> Self { Self::default() }

    pub fn with_params(game_id: i64, replay_to_command_nr: i32, coach: impl Into<String>) -> Self {
        Self {
            game_id,
            replay_to_command_nr,
            coach: Some(coach.into()),
        }
    }

    pub fn get_game_id(&self) -> i64 { self.game_id }
    pub fn get_replay_to_command_nr(&self) -> i32 { self.replay_to_command_nr }
    pub fn get_coach(&self) -> Option<&str> { self.coach.as_deref() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let cmd = ClientCommandReplay::with_params(42, 100, "coach1");
        assert_eq!(cmd.get_game_id(), 42);
        assert_eq!(cmd.get_replay_to_command_nr(), 100);
        assert_eq!(cmd.get_coach(), Some("coach1"));
    }

    #[test]
    fn default_is_zeroed() {
        let cmd = ClientCommandReplay::new();
        assert_eq!(cmd.game_id, 0);
        assert_eq!(cmd.replay_to_command_nr, 0);
        assert!(cmd.coach.is_none());
    }

    #[test]
    fn large_game_id_stored() {
        let cmd = ClientCommandReplay::with_params(i64::MAX, 0, "coach");
        assert_eq!(cmd.get_game_id(), i64::MAX);
    }
}
