/// Root-level abstract base for the PileDriver step sequence generator.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.PileDriver`.

#[derive(Debug, Clone, Default)]
pub struct PileDriverParams {
    pub target_player_id: Option<String>,
}

pub struct PileDriver;

impl PileDriver {
    pub fn new() -> Self { Self }
}

impl Default for PileDriver {
    fn default() -> Self { Self::new() }
}
