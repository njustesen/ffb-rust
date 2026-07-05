/// BB2025 select (activate player) step sequence — the action dispatch hub.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2025.Select`.
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};
use super::activation_sequence_builder::ActivationSequenceBuilder;

/// Parameters for the Select sequence — mirrors Java `Select.SequenceParams`.
#[derive(Debug, Clone, Default)]
pub struct SelectParams {
    pub update_persistence: bool,
    /// True when the current player action is a blitz move (affects ResetFumblerooskie).
    pub is_blitz_move: bool,
}

pub struct Select;

impl Select {
    pub fn new() -> Self { Self }

    /// Build the select step sequence (Java `pushSequence`).
    pub fn build_sequence(params: &SelectParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        let fl = labels::END_SELECTING;

        // 1 INIT_SELECTING
        seq.add(StepId::InitSelecting, vec![
            StepParameter::GotoLabelOnEnd(fl.into()),
            StepParameter::UpdatePersistence(params.update_persistence),
        ]);

        // 2 [ACTIVATION(END_SELECTING)] — no SET_DEFENDER for select
        ActivationSequenceBuilder::new()
            .with_failure_label(fl)
            .add_to(&mut seq);

        // 3 GOTO_LABEL → NEXT (alternate → END_SELECTING)
        seq.add(StepId::GotoLabel, vec![
            StepParameter::GotoLabel(labels::NEXT.into()),
            StepParameter::AlternateGotoLabel(fl.into()),
        ]);

        // 4 JUMP_UP [NEXT]
        seq.add_labelled(StepId::JumpUp, labels::NEXT, vec![
            StepParameter::GotoLabelOnFailure(fl.into()),
        ]);

        // 5 STAND_UP
        seq.add(StepId::StandUp, vec![
            StepParameter::GotoLabelOnFailure(fl.into()),
        ]);

        // 6 RESET_FUMBLEROOSKIE [END_SELECTING]
        seq.add_labelled(StepId::ResetFumblerooskie, fl, vec![
            StepParameter::InSelect(true),
            StepParameter::ResetForFailedBlock(params.is_blitz_move),
        ]);

        // 7 END_SELECTING (block_targets left empty — extended by skill hooks)
        seq.add(StepId::EndSelecting, vec![]);

        seq.build()
    }
}

impl Default for Select {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_sequence_starts_with_init_selecting() {
        let steps = Select::build_sequence(&SelectParams::default());
        assert_eq!(steps[0].step_id, StepId::InitSelecting);
    }

    #[test]
    fn select_sequence_ends_with_end_selecting() {
        let steps = Select::build_sequence(&SelectParams::default());
        let last = steps.last().unwrap();
        assert_eq!(last.step_id, StepId::EndSelecting);
    }

    #[test]
    fn select_sequence_has_reset_fumblerooskie_labelled_end_selecting() {
        let steps = Select::build_sequence(&SelectParams::default());
        let rfr = steps.iter().find(|s| s.step_id == StepId::ResetFumblerooskie).unwrap();
        assert_eq!(rfr.label.as_deref(), Some(labels::END_SELECTING));
    }

    #[test]
    fn update_persistence_param_passed_to_init_selecting() {
        let params = SelectParams { update_persistence: true, ..Default::default() };
        let steps = Select::build_sequence(&params);
        let has = steps[0].params.iter().any(|p| matches!(p, StepParameter::UpdatePersistence(true)));
        assert!(has);
    }

    #[test]
    fn is_blitz_move_sets_reset_for_failed_block() {
        let params = SelectParams { is_blitz_move: true, ..Default::default() };
        let steps = Select::build_sequence(&params);
        let rfr = steps.iter().find(|s| s.step_id == StepId::ResetFumblerooskie).unwrap();
        let has = rfr.params.iter().any(|p| matches!(p, StepParameter::ResetForFailedBlock(true)));
        assert!(has);
    }
}
