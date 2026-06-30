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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black_ink_params_default_no_label() {
        let p = BlackInkParams::default();
        assert!(p.go_to_label_failure.is_none());
    }

    #[test]
    fn black_ink_params_default_no_player_state() {
        let p = BlackInkParams::default();
        assert!(p.old_player_state.is_none());
    }

    #[test]
    fn black_ink_struct_is_default() {
        let _ = BlackInk::default();
    }
}
