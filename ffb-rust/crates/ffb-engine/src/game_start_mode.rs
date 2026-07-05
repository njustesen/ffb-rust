/// 1:1 translation of `com.fumbbl.ffb.server.GameStartMode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameStartMode {
    StartGame,
    StartTestGame,
    ScheduleGame,
}

impl GameStartMode {
    pub fn get_name(&self) -> &'static str {
        match self {
            GameStartMode::StartGame => "START GAME",
            GameStartMode::StartTestGame => "START TEST GAME",
            GameStartMode::ScheduleGame => "SCHEDULE GAME",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_game_name() {
        assert_eq!(GameStartMode::StartGame.get_name(), "START GAME");
    }

    #[test]
    fn start_test_game_name() {
        assert_eq!(GameStartMode::StartTestGame.get_name(), "START TEST GAME");
    }

    #[test]
    fn schedule_game_name() {
        assert_eq!(GameStartMode::ScheduleGame.get_name(), "SCHEDULE GAME");
    }
}
