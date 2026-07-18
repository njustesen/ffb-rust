/// BB2025 Black Ink step sequence (single-step).
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2025.BlackInk`.
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};
use super::activation_sequence_builder::ActivationSequenceBuilder;

#[derive(Debug, Clone, Default)]
pub struct BlackInkParams {
    pub failure_label: String,
    pub old_player_state: Option<ffb_model::enums::PlayerState>,
}

pub struct BlackInk;

impl BlackInk {
    pub fn new() -> Self { Self }

    pub fn build_sequence(params: &BlackInkParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();

        // 1-13 [ACTIVATION(END)]
        ActivationSequenceBuilder::new()
            .with_failure_label(labels::END)
            .add_to(&mut seq);

        // 14 BLACK_INK [END]
        let mut p = vec![StepParameter::GotoLabelOnFailure(params.failure_label.clone())];
        if let Some(state) = params.old_player_state {
            p.push(StepParameter::OldPlayerState(state));
        }
        seq.add_labelled(StepId::BlackInk, labels::END, p);
        seq.build()
    }
}

impl Default for BlackInk {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black_ink_last_step_labelled_end() {
        let steps = BlackInk::build_sequence(&BlackInkParams {
            failure_label: "X".into(),
            old_player_state: None,
        });
        let last = steps.last().unwrap();
        assert_eq!(last.step_id, StepId::BlackInk);
        assert_eq!(last.label.as_deref(), Some(labels::END));
    }

    #[test]
    fn activation_sub_sequence_precedes_black_ink() {
        // Java pushSequence: ActivationSequenceBuilder.create()...addTo(sequence) before BLACK_INK.
        let steps = BlackInk::build_sequence(&BlackInkParams::default());
        assert_eq!(steps.len(), 14);
        assert_eq!(steps[0].step_id, StepId::InitActivation);
    }

    #[test]
    fn failure_label_in_params() {
        let steps = BlackInk::build_sequence(&BlackInkParams {
            failure_label: "theLabel".into(),
            old_player_state: None,
        });
        let last = steps.last().unwrap();
        let has = last.params.iter().any(|p| {
            matches!(p, StepParameter::GotoLabelOnFailure(l) if l == "theLabel")
        });
        assert!(has);
    }

    #[test]
    fn old_player_state_added_when_some() {
        use ffb_model::enums::{PlayerState, PS_STANDING};
        let state = PlayerState::new(PS_STANDING);
        let steps = BlackInk::build_sequence(&BlackInkParams {
            failure_label: "X".into(),
            old_player_state: Some(state),
        });
        let last = steps.last().unwrap();
        let has = last.params.iter().any(|p| matches!(p, StepParameter::OldPlayerState(_)));
        assert!(has);
    }

    #[test]
    fn params_with_fields_set() {
        let p = BlackInkParams {
            failure_label: "myLabel".into(),
            old_player_state: None,
        };
        assert_eq!(p.failure_label, "myLabel");
        assert!(p.old_player_state.is_none());
    }

    #[test]
    fn params_clone() {
        let p = BlackInkParams { failure_label: "lbl".into(), old_player_state: None };
        let q = p.clone();
        assert_eq!(q.failure_label, "lbl");
    }
}
