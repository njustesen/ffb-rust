/// 1:1 translation of com.fumbbl.ffb.server.net.commands.InternalServerCommandReplayLoaded.
use super::internal_server_command::InternalServerCommand;

pub struct InternalServerCommandReplayLoaded {
    pub game_id: i64,
    pub replay_to_command_nr: i32,
    pub coach: String,
}

impl InternalServerCommandReplayLoaded {
    pub fn new(game_id: i64, replay_to_command_nr: i32, coach: String) -> Self {
        Self { game_id, replay_to_command_nr, coach }
    }

    pub fn get_replay_to_command_nr(&self) -> i32 {
        self.replay_to_command_nr
    }

    pub fn get_coach(&self) -> &str {
        &self.coach
    }
}

impl InternalServerCommand for InternalServerCommandReplayLoaded {
    fn get_id(&self) -> &'static str {
        "internalServerReplayLoaded"
    }

    fn get_game_id(&self) -> i64 {
        self.game_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let _ = InternalServerCommandReplayLoaded::new(1, 50, "coach".to_string());
    }

    #[test]
    fn get_id() {
        let c = InternalServerCommandReplayLoaded::new(1, 0, "c".to_string());
        assert_eq!(c.get_id(), "internalServerReplayLoaded");
    }

    #[test]
    fn get_game_id() {
        let c = InternalServerCommandReplayLoaded::new(7, 0, "c".to_string());
        assert_eq!(c.get_game_id(), 7);
    }

    #[test]
    fn get_replay_to_command_nr() {
        let c = InternalServerCommandReplayLoaded::new(1, 42, "c".to_string());
        assert_eq!(c.get_replay_to_command_nr(), 42);
    }

    #[test]
    fn is_internal() {
        let c = InternalServerCommandReplayLoaded::new(1, 0, "c".to_string());
        assert!(c.is_internal());
    }
}
