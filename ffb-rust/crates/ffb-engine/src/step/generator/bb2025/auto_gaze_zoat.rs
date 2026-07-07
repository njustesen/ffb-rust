/// BB2025 Auto Gaze Zoat step sequence (single-step).
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2025.AutoGazeZoat`.
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

#[derive(Debug, Clone, Default)]
pub struct AutoGazeZoatParams {
    pub failure_label: String,
    pub old_player_state: Option<ffb_model::enums::PlayerState>,
}

pub struct AutoGazeZoat;

impl AutoGazeZoat {
    pub fn new() -> Self { Self }

    pub fn build_sequence(params: &AutoGazeZoatParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        let mut p = vec![StepParameter::GotoLabelOnFailure(params.failure_label.clone())];
        if let Some(state) = params.old_player_state {
            p.push(StepParameter::OldPlayerState(state));
        }
        seq.add_labelled(StepId::AutoGazeZoat, labels::END, p);
        seq.build()
    }
}

impl Default for AutoGazeZoat {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_gaze_zoat_single_step_labelled_end() {
        let steps = AutoGazeZoat::build_sequence(&AutoGazeZoatParams {
            failure_label: "someLabel".into(),
            old_player_state: None,
        });
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].step_id, StepId::AutoGazeZoat);
        assert_eq!(steps[0].label.as_deref(), Some(labels::END));
    }

    #[test]
    fn failure_label_passed_as_goto_label_on_failure() {
        let steps = AutoGazeZoat::build_sequence(&AutoGazeZoatParams {
            failure_label: "myLabel".into(),
            old_player_state: None,
        });
        let has_label = steps[0].params.iter().any(|p| {
            matches!(p, StepParameter::GotoLabelOnFailure(l) if l == "myLabel")
        });
        assert!(has_label);
    }

    #[test]
    fn old_player_state_added_when_some() {
        use ffb_model::enums::{PlayerState, PS_STANDING};
        let state = PlayerState::new(PS_STANDING);
        let steps = AutoGazeZoat::build_sequence(&AutoGazeZoatParams {
            failure_label: "X".into(),
            old_player_state: Some(state),
        });
        let has_state = steps[0].params.iter().any(|p| matches!(p, StepParameter::OldPlayerState(_)));
        assert!(has_state);
    }

    #[test]
    fn old_player_state_absent_when_none() {
        let steps = AutoGazeZoat::build_sequence(&AutoGazeZoatParams {
            failure_label: "X".into(),
            old_player_state: None,
        });
        let has_state = steps[0].params.iter().any(|p| matches!(p, StepParameter::OldPlayerState(_)));
        assert!(!has_state);
    }
    #[test]
    fn build_sequence_is_nonempty() {
        assert!(!AutoGazeZoat::build_sequence(&Default::default()).is_empty());
    }

}
