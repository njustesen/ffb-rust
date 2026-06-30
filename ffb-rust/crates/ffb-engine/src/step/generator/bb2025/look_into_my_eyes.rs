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
}
