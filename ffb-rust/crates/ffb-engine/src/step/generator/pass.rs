/// Root-level abstract base for the Pass step sequence generator.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.Pass`.
use ffb_model::types::FieldCoordinate;

#[derive(Debug, Clone, Default)]
pub struct PassParams {
    pub target_coordinate: Option<FieldCoordinate>,
}

pub struct Pass;

impl Pass {
    pub fn new() -> Self { Self }
}

impl Default for Pass {
    fn default() -> Self { Self::new() }
}
