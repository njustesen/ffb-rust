/// Root-level abstract base for the AutoGazeZoat step sequence generator.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.AutoGazeZoat`.
use ffb_model::enums::PlayerState;

#[derive(Debug, Clone)]
pub struct AutoGazeZoatParams {
    pub go_to_label_failure: Option<String>,
    pub old_player_state: Option<PlayerState>,
}

impl Default for AutoGazeZoatParams {
    fn default() -> Self {
        Self {
            go_to_label_failure: None,
            old_player_state: None,
        }
    }
}

pub struct AutoGazeZoat;

impl AutoGazeZoat {
    pub fn new() -> Self { Self }
}

impl Default for AutoGazeZoat {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_gaze_zoat_params_default_no_label() {
        let p = AutoGazeZoatParams::default();
        assert!(p.go_to_label_failure.is_none());
    }

    #[test]
    fn auto_gaze_zoat_params_default_no_player_state() {
        let p = AutoGazeZoatParams::default();
        assert!(p.old_player_state.is_none());
    }

    #[test]
    fn auto_gaze_zoat_struct_is_default() {
        let _ = AutoGazeZoat::default();
    }
}
