/// Root-level abstract base for the BlitzMove step sequence generator.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.BlitzMove`.
use ffb_model::types::FieldCoordinate;

#[derive(Debug, Clone, Default)]
pub struct BlitzMoveParams {
    pub move_stack: Vec<FieldCoordinate>,
    pub gaze_victim_id: Option<String>,
    pub move_start: Option<FieldCoordinate>,
}

pub struct BlitzMove;

impl BlitzMove {
    pub fn new() -> Self { Self }
}

impl Default for BlitzMove {
    fn default() -> Self { Self::new() }
}
