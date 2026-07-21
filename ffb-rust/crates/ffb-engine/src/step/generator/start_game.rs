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
