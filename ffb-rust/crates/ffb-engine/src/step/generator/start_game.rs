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
}
