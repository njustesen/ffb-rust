/// Root-level abstract base for the Move step sequence generator.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.Move`.
use ffb_model::enums::PlayerAction;
use ffb_model::types::FieldCoordinate;

#[derive(Debug, Clone, Default)]
pub struct MoveParams {
    pub move_stack: Vec<FieldCoordinate>,
    pub gaze_victim_id: Option<String>,
    pub move_start: Option<FieldCoordinate>,
    pub ball_and_chain_rr_setting: Option<String>,
    pub bloodlust_action: Option<PlayerAction>,
}

pub struct Move;

impl Move {
    pub fn new() -> Self { Self }
}

impl Default for Move {
    fn default() -> Self { Self::new() }
}
