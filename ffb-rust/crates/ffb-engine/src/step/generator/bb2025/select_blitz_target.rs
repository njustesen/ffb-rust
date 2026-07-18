/// BB2025 select-blitz-target step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2025.SelectBlitzTarget`.
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};
use super::activation_sequence_builder::ActivationSequenceBuilder;

pub struct SelectBlitzTarget;

impl SelectBlitzTarget {
    pub fn new() -> Self { Self }

    pub fn build_sequence() -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        // 1 SELECT_BLITZ_TARGET [SELECT]
        seq.add_labelled(StepId::SelectBlitzTarget, labels::SELECT, vec![
            StepParameter::GotoLabelOnEnd(labels::END_BLITZING.into()),
        ]);

        // [ACTIVATION(END_BLITZING)]
        ActivationSequenceBuilder::new()
            .with_failure_label(labels::END_BLITZING)
            .add_to(&mut seq);

        // 2 JUMP_UP
        seq.add(StepId::JumpUp, vec![
            StepParameter::GotoLabelOnFailure(labels::END_BLITZING.into()),
        ]);
        // 3 STAND_UP
        seq.add(StepId::StandUp, vec![
            StepParameter::GotoLabelOnFailure(labels::END_BLITZING.into()),
        ]);
        // 4 SELECT_BLITZ_TARGET_END [END_BLITZING]
        seq.add_labelled(StepId::SelectBlitzTargetEnd, labels::END_BLITZING, vec![]);
        seq.build()
    }
}

impl Default for SelectBlitzTarget {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_blitz_target_has_17_steps_with_activation() {
        // Java pushSequence: SELECT_BLITZ_TARGET (1) + ActivationSequenceBuilder.create()...addTo(sequence)
        // (13) + 3 own steps = 17.
        let steps = SelectBlitzTarget::build_sequence();
        assert_eq!(steps.len(), 17);
        assert_eq!(steps[1].step_id, StepId::InitActivation);
    }

    #[test]
    fn select_blitz_target_is_labelled_select() {
        let steps = SelectBlitzTarget::build_sequence();
        let s = steps.iter().find(|s| s.step_id == StepId::SelectBlitzTarget).unwrap();
        assert_eq!(s.label.as_deref(), Some(labels::SELECT));
    }

    #[test]
    fn select_blitz_target_end_is_labelled_end_blitzing() {
        let steps = SelectBlitzTarget::build_sequence();
        let s = steps.iter().find(|s| s.step_id == StepId::SelectBlitzTargetEnd).unwrap();
        assert_eq!(s.label.as_deref(), Some(labels::END_BLITZING));
    }

    #[test]
    fn select_blitz_target_has_jump_up_and_stand_up() {
        let steps = SelectBlitzTarget::build_sequence();
        assert!(steps.iter().any(|s| s.step_id == StepId::JumpUp));
        assert!(steps.iter().any(|s| s.step_id == StepId::StandUp));
    }
    #[test]
    fn build_sequence_returns_vec() {
        let seq = SelectBlitzTarget::build_sequence();
        assert!(!seq.is_empty(), "sequence should not be empty");
    }

}
