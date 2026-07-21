/// Root-level abstract base for the EndGame step sequence generator.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.EndGame`.

#[derive(Debug, Clone, Default)]
pub struct EndGameParams {
    pub admin_mode: bool,
}

pub struct EndGame;

impl EndGame {
    pub fn new() -> Self { Self }
}

impl Default for EndGame {
    fn default() -> Self { Self::new() }
}
