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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blitz_move_params_default_empty_stack() {
        let p = BlitzMoveParams::default();
        assert!(p.move_stack.is_empty());
    }

    #[test]
    fn blitz_move_params_default_no_gaze_victim() {
        let p = BlitzMoveParams::default();
        assert!(p.gaze_victim_id.is_none());
    }

    #[test]
    fn blitz_move_params_default_no_move_start() {
        let p = BlitzMoveParams::default();
        assert!(p.move_start.is_none());
    }

    #[test]
    fn params_with_fields_set() {
        let p = BlitzMoveParams {
            gaze_victim_id: Some("victim".into()),
            ..Default::default()
        };
        assert_eq!(p.gaze_victim_id.as_deref(), Some("victim"));
        assert!(p.move_stack.is_empty());
    }

    #[test]
    fn params_clone() {
        let p = BlitzMoveParams { gaze_victim_id: Some("x".into()), ..Default::default() };
        let q = p.clone();
        assert_eq!(q.gaze_victim_id.as_deref(), Some("x"));
    }
}
