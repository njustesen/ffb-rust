/// BB2025 Look Into My Eyes step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2025.LookIntoMyEyes`.
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

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
        // 1 INIT_LOOK_INTO_MY_EYES
        seq.add(StepId::InitLookIntoMyEyes, vec![]);
        // 2 FOUL_APPEARANCE (goto END on failure)
        seq.add(StepId::FoulAppearance, vec![
            StepParameter::GotoLabelOnFailure(labels::END.into()),
        ]);
        // 3 LOOK_INTO_MY_EYES [END]
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
    fn look_into_my_eyes_has_3_steps() {
        let steps = LookIntoMyEyes::build_sequence(&LookIntoMyEyesParams::default());
        assert_eq!(steps.len(), 3);
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
    fn first_step_is_init_look_into_my_eyes() {
        let steps = LookIntoMyEyes::build_sequence(&LookIntoMyEyesParams::default());
        assert_eq!(steps[0].step_id, StepId::InitLookIntoMyEyes);
    }
}
