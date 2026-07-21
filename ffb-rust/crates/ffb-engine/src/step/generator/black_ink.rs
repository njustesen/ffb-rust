/// Root-level abstract base for the BlackInk step sequence generator.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.BlackInk`.
use ffb_model::enums::PlayerState;

#[derive(Debug, Clone)]
pub struct BlackInkParams {
    pub go_to_label_failure: Option<String>,
    pub old_player_state: Option<PlayerState>,
}

impl Default for BlackInkParams {
    fn default() -> Self {
        Self {
            go_to_label_failure: None,
            old_player_state: None,
        }
    }
}

pub struct BlackInk;

impl BlackInk {
    pub fn new() -> Self { Self }
}

impl Default for BlackInk {
    fn default() -> Self { Self::new() }
}
