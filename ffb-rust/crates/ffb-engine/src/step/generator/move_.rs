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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_params_default_empty_stack() {
        let p = MoveParams::default();
        assert!(p.move_stack.is_empty());
    }

    #[test]
    fn move_params_default_no_gaze_victim() {
        let p = MoveParams::default();
        assert!(p.gaze_victim_id.is_none());
    }

    #[test]
    fn move_params_default_no_bloodlust_action() {
        let p = MoveParams::default();
        assert!(p.bloodlust_action.is_none());
    }

    #[test]
    fn params_with_fields_set() {
        let p = MoveParams {
            gaze_victim_id: Some("victim".into()),
            ball_and_chain_rr_setting: Some("setting".into()),
            ..Default::default()
        };
        assert_eq!(p.gaze_victim_id.as_deref(), Some("victim"));
        assert_eq!(p.ball_and_chain_rr_setting.as_deref(), Some("setting"));
    }

    #[test]
    fn params_clone() {
        let p = MoveParams { gaze_victim_id: Some("x".into()), ..Default::default() };
        let q = p.clone();
        assert_eq!(q.gaze_victim_id.as_deref(), Some("x"));
    }
}
