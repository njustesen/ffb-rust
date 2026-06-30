/// BB2020 multi-block step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2020.MultiBlock`.
use ffb_model::enums::ApothecaryMode;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

#[derive(Debug, Clone, Default)]
pub struct MultiBlockParams {
    pub block_targets: Vec<String>,
}

pub struct MultiBlock;

impl MultiBlock {
    pub fn new() -> Self { Self }

    pub fn build_sequence(params: &MultiBlockParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        let fl = labels::END_BLOCKING;
        let size = params.block_targets.len() as i32;

        // ACTIVATION BLOCK (with GotoLabel, BloodLust has failure label END_BLOCKING)
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

        // FOUL_APPEARANCE_MULTIPLE
        seq.add(StepId::FoulAppearanceMultiple, vec![
            StepParameter::GotoLabelOnFailure(labels::END_BLOCKING.into()),
            StepParameter::BlockTargets(params.block_targets.clone()),
        ]);
        // DISPATCH_DUMP_OFF
        seq.add(StepId::DispatchDumpOff, vec![
            StepParameter::BlockTargets(params.block_targets.clone()),
        ]);
        // BLOCK_STATISTICS
        seq.add(StepId::BlockStatistics, vec![StepParameter::Increment(size)]);
        // MULTIPLE_BLOCK_FORK
        seq.add(StepId::MultipleBlockFork, vec![
            StepParameter::BlockTargets(params.block_targets.clone()),
        ]);
        // PLACE_BALL
        seq.add(StepId::PlaceBall, vec![]);
        // APOTHECARY_MULTIPLE (acting_team = false)
        seq.add(StepId::ApothecaryMultiple, vec![StepParameter::ActingTeam(false)]);
        // APOTHECARY_MULTIPLE (acting_team = true)
        seq.add(StepId::ApothecaryMultiple, vec![StepParameter::ActingTeam(true)]);
        // CATCH_SCATTER_THROW_IN [SCATTER_BALL]
        seq.add_labelled(StepId::CatchScatterThrowIn, labels::SCATTER_BALL, vec![]);
        // END_BLOCKING [END_BLOCKING]
        seq.add_labelled(StepId::EndBlocking, labels::END_BLOCKING, vec![]);

        seq.build()
    }
}

impl Default for MultiBlock {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multi_block_has_activation_block() {
        let steps = MultiBlock::build_sequence(&MultiBlockParams::default());
        assert!(steps.iter().any(|s| s.step_id == StepId::InitActivation));
        assert!(steps.iter().any(|s| s.step_id == StepId::BoneHead));
    }

    #[test]
    fn multi_block_ends_with_end_blocking_labelled() {
        let steps = MultiBlock::build_sequence(&MultiBlockParams::default());
        let last = steps.last().unwrap();
        assert_eq!(last.step_id, StepId::EndBlocking);
        assert_eq!(last.label.as_deref(), Some(labels::END_BLOCKING));
    }

    #[test]
    fn multi_block_catch_scatter_labelled_scatter_ball() {
        let steps = MultiBlock::build_sequence(&MultiBlockParams::default());
        let cs = steps.iter().find(|s| s.label.as_deref() == Some(labels::SCATTER_BALL)).unwrap();
        assert_eq!(cs.step_id, StepId::CatchScatterThrowIn);
    }

    #[test]
    fn multi_block_apothecary_multiple_both() {
        let steps = MultiBlock::build_sequence(&MultiBlockParams::default());
        let apos: Vec<_> = steps.iter().filter(|s| s.step_id == StepId::ApothecaryMultiple).collect();
        assert_eq!(apos.len(), 2);
    }

    #[test]
    fn multi_block_blood_lust_has_failure_label_end_blocking() {
        let steps = MultiBlock::build_sequence(&MultiBlockParams::default());
        let bl = steps.iter().find(|s| s.step_id == StepId::BloodLust).unwrap();
        assert!(bl.params.iter().any(|p| matches!(p, StepParameter::GotoLabelOnFailure(_))));
    }
}
