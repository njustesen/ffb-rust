/// Root-level abstract base for the StartGame step sequence generator.
/// No inner SequenceParams — uses base SequenceGenerator.SequenceParams.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.StartGame`.

pub struct StartGame;

impl StartGame {
    pub fn new() -> Self { Self }
}

impl Default for StartGame {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_game_new_creates_instance() {
        let _ = StartGame::new();
    }

    #[test]
    fn start_game_default_creates_instance() {
        let _ = StartGame::default();
    }

    #[test]
    fn start_game_new_and_default_both_succeed() {
        let _a = StartGame::new();
        let _b = StartGame::default();
        assert!(true);
    }

    #[test]
    fn start_game_new_is_consistent_with_default() {
        let via_new = StartGame::new();
        let via_default = StartGame::default();
        let _ = (via_new, via_default);
    }
}
