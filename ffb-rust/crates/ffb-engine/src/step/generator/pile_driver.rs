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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pile_driver_params_default_no_target() {
        let p = PileDriverParams::default();
        assert!(p.target_player_id.is_none());
    }

    #[test]
    fn pile_driver_params_can_set_target() {
        let p = PileDriverParams { target_player_id: Some("player-1".to_string()) };
        assert_eq!(p.target_player_id.as_deref(), Some("player-1"));
    }

    #[test]
    fn pile_driver_struct_is_default() {
        let _ = PileDriver::default();
    }
}
