/// BB2025 Auto Gaze Zoat step sequence (single-step).
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2025.AutoGazeZoat`.
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};
use super::activation_sequence_builder::ActivationSequenceBuilder;

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

        // 1-13 [ACTIVATION(END)]
        ActivationSequenceBuilder::new()
            .with_failure_label(labels::END)
            .add_to(&mut seq);

        // 14 AUTO_GAZE_ZOAT [END]
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
    fn auto_gaze_zoat_last_step_labelled_end() {
        let steps = AutoGazeZoat::build_sequence(&AutoGazeZoatParams {
            failure_label: "someLabel".into(),
            old_player_state: None,
        });
        let last = steps.last().unwrap();
        assert_eq!(last.step_id, StepId::AutoGazeZoat);
        assert_eq!(last.label.as_deref(), Some(labels::END));
    }

    #[test]
    fn activation_sub_sequence_precedes_auto_gaze_zoat() {
        // Java pushSequence: ActivationSequenceBuilder.create()...addTo(sequence) before AUTO_GAZE_ZOAT.
        let steps = AutoGazeZoat::build_sequence(&AutoGazeZoatParams::default());
        assert_eq!(steps.len(), 14);
        assert_eq!(steps[0].step_id, StepId::InitActivation);
        assert_eq!(steps[13].step_id, StepId::AutoGazeZoat);
    }

    #[test]
    fn failure_label_passed_as_goto_label_on_failure() {
        let steps = AutoGazeZoat::build_sequence(&AutoGazeZoatParams {
            failure_label: "myLabel".into(),
            old_player_state: None,
        });
        let last = steps.last().unwrap();
        let has_label = last.params.iter().any(|p| {
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
        let last = steps.last().unwrap();
        let has_state = last.params.iter().any(|p| matches!(p, StepParameter::OldPlayerState(_)));
        assert!(has_state);
    }

    #[test]
    fn old_player_state_absent_when_none() {
        let steps = AutoGazeZoat::build_sequence(&AutoGazeZoatParams {
            failure_label: "X".into(),
            old_player_state: None,
        });
        let last = steps.last().unwrap();
        let has_state = last.params.iter().any(|p| matches!(p, StepParameter::OldPlayerState(_)));
        assert!(!has_state);
    }
    #[test]
    fn build_sequence_is_nonempty() {
        assert!(!AutoGazeZoat::build_sequence(&Default::default()).is_empty());
    }

}
