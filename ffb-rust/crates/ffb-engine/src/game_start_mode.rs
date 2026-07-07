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

    #[test]
    fn all_variants_are_distinct() {
        assert_ne!(GameStartMode::StartGame, GameStartMode::StartTestGame);
        assert_ne!(GameStartMode::StartGame, GameStartMode::ScheduleGame);
        assert_ne!(GameStartMode::StartTestGame, GameStartMode::ScheduleGame);
    }

    #[test]
    fn copy_semantics_preserved() {
        let a = GameStartMode::ScheduleGame;
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn clone_equals_original() {
        let a = GameStartMode::StartTestGame;
        assert_eq!(a.clone(), a);
    }

    #[test]
    fn debug_format_contains_variant_name() {
        let s = format!("{:?}", GameStartMode::StartGame);
        assert!(s.contains("StartGame"));
        let s2 = format!("{:?}", GameStartMode::ScheduleGame);
        assert!(s2.contains("ScheduleGame"));
    }

    #[test]
    fn get_name_returns_static_str() {
        // Verify the return type is &'static str by binding to it.
        let name: &'static str = GameStartMode::StartGame.get_name();
        assert!(!name.is_empty());
    }
}
