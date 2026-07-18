/// BB2025 Look Into My Eyes step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2025.LookIntoMyEyes`.
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};
use super::activation_sequence_builder::ActivationSequenceBuilder;

#[derive(Debug, Clone, Default)]
pub struct LookIntoMyEyesParams {
    pub push_select: bool,
    pub goto_on_end: String,
}

pub struct LookIntoMyEyes;

impl LookIntoMyEyes {
    pub fn new() -> Self { Self }

    pub fn build_sequence(params: &LookIntoMyEyesParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();

        // 1-13 [ACTIVATION(END)]
        ActivationSequenceBuilder::new()
            .with_failure_label(labels::END)
            .add_to(&mut seq);

        // 14 INIT_LOOK_INTO_MY_EYES
        seq.add(StepId::InitLookIntoMyEyes, vec![]);
        // 15 FOUL_APPEARANCE (goto END on failure)
        seq.add(StepId::FoulAppearance, vec![
            StepParameter::GotoLabelOnFailure(labels::END.into()),
        ]);
        // 16 LOOK_INTO_MY_EYES [END]
        seq.add_labelled(StepId::LookIntoMyEyes, labels::END, vec![
            StepParameter::PushSelect(params.push_select),
            StepParameter::GotoLabelOnEnd(params.goto_on_end.clone()),
        ]);
        seq.build()
    }
}

impl Default for LookIntoMyEyes {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn look_into_my_eyes_has_16_steps_with_activation() {
        // Java pushSequence: ActivationSequenceBuilder.create()...addTo(sequence) (13) +
        // INIT_LOOK_INTO_MY_EYES + FOUL_APPEARANCE + LOOK_INTO_MY_EYES (3) = 16.
        let steps = LookIntoMyEyes::build_sequence(&LookIntoMyEyesParams::default());
        assert_eq!(steps.len(), 16);
        assert_eq!(steps[0].step_id, StepId::InitActivation);
    }

    #[test]
    fn look_into_my_eyes_step_is_labelled_end() {
        let steps = LookIntoMyEyes::build_sequence(&LookIntoMyEyesParams::default());
        let s = steps.iter().find(|s| s.step_id == StepId::LookIntoMyEyes).unwrap();
        assert_eq!(s.label.as_deref(), Some(labels::END));
    }

    #[test]
    fn push_select_param_wired() {
        let steps = LookIntoMyEyes::build_sequence(&LookIntoMyEyesParams { push_select: true, goto_on_end: String::new() });
        let s = steps.iter().find(|s| s.step_id == StepId::LookIntoMyEyes).unwrap();
        assert!(s.params.iter().any(|p| matches!(p, StepParameter::PushSelect(true))));
    }

    #[test]
    fn goto_on_end_wired() {
        let steps = LookIntoMyEyes::build_sequence(&LookIntoMyEyesParams { push_select: false, goto_on_end: "MY_LABEL".into() });
        let s = steps.iter().find(|s| s.step_id == StepId::LookIntoMyEyes).unwrap();
        assert!(s.params.iter().any(|p| matches!(p, StepParameter::GotoLabelOnEnd(l) if l == "MY_LABEL")));
    }

    #[test]
    fn init_look_into_my_eyes_follows_activation_sub_sequence() {
        let steps = LookIntoMyEyes::build_sequence(&LookIntoMyEyesParams::default());
        assert_eq!(steps[13].step_id, StepId::InitLookIntoMyEyes);
    }
}
