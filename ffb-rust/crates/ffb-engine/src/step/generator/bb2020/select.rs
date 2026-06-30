/// BB2020 select (activate player) step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2020.Select`.
use ffb_model::enums::ApothecaryMode;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

#[derive(Debug, Clone, Default)]
pub struct SelectParams {
    pub update_persistence: bool,
    pub is_blitz_move: bool,
    pub block_targets: Vec<String>,
}

pub struct Select;

impl Select {
    pub fn new() -> Self { Self }

    pub fn build_sequence(params: &SelectParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        let fl = labels::END_SELECTING;

        // 1 INIT_SELECTING
        seq.add(StepId::InitSelecting, vec![
            StepParameter::GotoLabelOnEnd(fl.into()),
            StepParameter::UpdatePersistence(params.update_persistence),
        ]);

        // 2-13 ACTIVATION BLOCK (with GotoLabel, BloodLust → fl)
        let fl_s: String = fl.into();
        seq.add(StepId::InitActivation, vec![]);
        seq.add(StepId::AnimalSavagery, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);
        seq.add(StepId::HandleDropPlayerContext, vec![]);
        seq.add(StepId::PlaceBall, vec![]);
        seq.add(StepId::Apothecary, vec![StepParameter::ApothecaryMode(ApothecaryMode::AnimalSavagery)]);
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        seq.add(StepId::GotoLabel, vec![
            StepParameter::GotoLabel(labels::NEXT.into()),
            StepParameter::AlternateGotoLabel(fl_s.clone()),
        ]);
        seq.add_labelled(StepId::BoneHead, labels::NEXT, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);
        seq.add(StepId::ReallyStupid, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);
        seq.add(StepId::TakeRoot, vec![]);
        seq.add(StepId::UnchannelledFury, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);
        seq.add(StepId::BloodLust, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);

        // After activation: extra GotoLabel to NEXT (alternate = END_SELECTING)
        seq.add(StepId::GotoLabel, vec![
            StepParameter::GotoLabel(labels::NEXT.into()),
            StepParameter::AlternateGotoLabel(fl.into()),
        ]);
        // JUMP_UP [NEXT]
        seq.add_labelled(StepId::JumpUp, labels::NEXT, vec![
            StepParameter::GotoLabelOnFailure(fl.into()),
        ]);
        // STAND_UP
        seq.add(StepId::StandUp, vec![StepParameter::GotoLabelOnFailure(fl.into())]);
        // RESET_FUMBLEROOSKIE [END_SELECTING]
        seq.add_labelled(StepId::ResetFumblerooskie, fl, vec![
            StepParameter::InSelect(true),
            StepParameter::ResetForFailedBlock(params.is_blitz_move),
        ]);
        // END_SELECTING (with block_targets)
        if params.block_targets.is_empty() {
            seq.add(StepId::EndSelecting, vec![]);
        } else {
            seq.add(StepId::EndSelecting, vec![
                StepParameter::BlockTargets(params.block_targets.clone()),
            ]);
        }

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
    fn select_starts_with_init_selecting() {
        let steps = Select::build_sequence(&SelectParams::default());
        assert_eq!(steps[0].step_id, StepId::InitSelecting);
    }

    #[test]
    fn select_ends_with_end_selecting() {
        let steps = Select::build_sequence(&SelectParams::default());
        assert_eq!(steps.last().unwrap().step_id, StepId::EndSelecting);
    }

    #[test]
    fn select_has_activation_block() {
        let steps = Select::build_sequence(&SelectParams::default());
        assert!(steps.iter().any(|s| s.step_id == StepId::InitActivation));
    }

    #[test]
    fn select_reset_fumblerooskie_labelled_end_selecting() {
        let steps = Select::build_sequence(&SelectParams::default());
        let rfr = steps.iter().find(|s| s.step_id == StepId::ResetFumblerooskie).unwrap();
        assert_eq!(rfr.label.as_deref(), Some(labels::END_SELECTING));
    }

    #[test]
    fn select_jump_up_is_labelled_next() {
        let steps = Select::build_sequence(&SelectParams::default());
        let ju = steps.iter().find(|s| s.step_id == StepId::JumpUp).unwrap();
        assert_eq!(ju.label.as_deref(), Some(labels::NEXT));
    }
}
