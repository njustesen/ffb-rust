/// Root-level abstract base for the Kickoff step sequence generator.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.Kickoff`.

#[derive(Debug, Clone, Default)]
pub struct KickoffParams {
    pub with_coin_choice: bool,
}

pub struct Kickoff;

impl Kickoff {
    pub fn new() -> Self { Self }
}

impl Default for Kickoff {
    fn default() -> Self { Self::new() }
}
