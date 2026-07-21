/// Root-level abstract base for the ScatterPlayer step sequence generator.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.ScatterPlayer`.
use ffb_model::enums::PlayerState;
use ffb_model::types::FieldCoordinate;

#[derive(Debug, Clone)]
pub struct ScatterPlayerParams {
    pub thrown_player_id: Option<String>,
    pub thrown_player_state: Option<PlayerState>,
    pub thrown_player_coordinate: Option<FieldCoordinate>,
    pub has_swoop: bool,
    pub thrown_player_has_ball: bool,
    pub throw_scatter: bool,
    pub deviate: bool,
    pub crash_landing: bool,
}

impl Default for ScatterPlayerParams {
    fn default() -> Self {
        Self {
            thrown_player_id: None,
            thrown_player_state: None,
            thrown_player_coordinate: None,
            has_swoop: false,
            thrown_player_has_ball: false,
            throw_scatter: false,
            deviate: false,
            crash_landing: false,
        }
    }
}

pub struct ScatterPlayer;

impl ScatterPlayer {
    pub fn new() -> Self { Self }
}

impl Default for ScatterPlayer {
    fn default() -> Self { Self::new() }
}
