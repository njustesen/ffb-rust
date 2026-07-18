/// BB2025 multi-block step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2025.MultiBlock`.
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};
use super::activation_sequence_builder::ActivationSequenceBuilder;

#[derive(Debug, Clone, Default)]
pub struct MultiBlockParams {
    pub block_targets: Vec<String>,
}

pub struct MultiBlock;

impl MultiBlock {
    pub fn new() -> Self { Self }

    pub fn build_sequence(params: &MultiBlockParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        let size = params.block_targets.len() as i32;
        // 0 [ACTIVATION(END_BLOCKING)]
        ActivationSequenceBuilder::new()
            .with_failure_label(labels::END_BLOCKING)
            .add_to(&mut seq);
        // 1 FOUL_APPEARANCE_MULTIPLE (fail → END_BLOCKING)
        seq.add(StepId::FoulAppearanceMultiple, vec![
            StepParameter::GotoLabelOnFailure(labels::END_BLOCKING.into()),
            StepParameter::BlockTargets(params.block_targets.clone()),
        ]);
        // 2 DISPATCH_DUMP_OFF
        seq.add(StepId::DispatchDumpOff, vec![
            StepParameter::BlockTargets(params.block_targets.clone()),
        ]);
        // 3 BLOCK_STATISTICS (increment = block_targets.len)
        seq.add(StepId::BlockStatistics, vec![
            StepParameter::Increment(size),
        ]);
        // 4 MULTI_BLOCK_FORK
        seq.add(StepId::MultipleBlockFork, vec![
            StepParameter::BlockTargets(params.block_targets.clone()),
        ]);
        // 5 PLACE_BALL
        seq.add(StepId::PlaceBall, vec![]);
        // 6 APOTHECARY_MULTIPLE (acting_team = false)
        seq.add(StepId::ApothecaryMultiple, vec![
            StepParameter::ActingTeam(false),
        ]);
        // 7 APOTHECARY_MULTIPLE (acting_team = true)
        seq.add(StepId::ApothecaryMultiple, vec![
            StepParameter::ActingTeam(true),
        ]);
        // 8 CATCH_SCATTER_THROW_IN [SCATTER_BALL]
        seq.add_labelled(StepId::CatchScatterThrowIn, labels::SCATTER_BALL, vec![]);
        // 9 END_BLOCKING [END_BLOCKING]
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
    fn multi_block_has_22_steps() {
        let steps = MultiBlock::build_sequence(&MultiBlockParams {
            block_targets: vec!["p1".into(), "p2".into()],
        });
        // 13 activation steps + 9 multi-block steps
        assert_eq!(steps.len(), 22);
    }

    #[test]
    fn multi_block_empty_targets_has_22_steps() {
        let steps = MultiBlock::build_sequence(&MultiBlockParams::default());
        assert_eq!(steps.len(), 22);
    }

    #[test]
    fn multi_block_starts_with_activation_sequence() {
        // Java: `ActivationSequenceBuilder.create().withFailureLabel(END_BLOCKING).addTo(sequence)`
        // is called before FOUL_APPEARANCE_MULTIPLE (MultiBlock.java:29-32).
        let steps = MultiBlock::build_sequence(&MultiBlockParams::default());
        assert_eq!(steps[0].step_id, StepId::InitActivation);
        assert!(steps.iter().any(|s| s.step_id == StepId::BoneHead));
        assert!(steps.iter().any(|s| s.step_id == StepId::ReallyStupid));
        assert!(steps.iter().any(|s| s.step_id == StepId::TakeRoot));
        assert!(steps.iter().any(|s| s.step_id == StepId::UnchannelledFury));
        assert!(steps.iter().any(|s| s.step_id == StepId::BloodLust));
        assert!(steps.iter().any(|s| s.step_id == StepId::AnimalSavagery));
        // FoulAppearanceMultiple must come after the activation sequence.
        let init_pos = steps.iter().position(|s| s.step_id == StepId::InitActivation).unwrap();
        let foul_pos = steps.iter().position(|s| s.step_id == StepId::FoulAppearanceMultiple).unwrap();
        assert!(init_pos < foul_pos);
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
    fn multi_block_apothecary_multiple_acting_team_both() {
        let steps = MultiBlock::build_sequence(&MultiBlockParams::default());
        let apos: Vec<_> = steps.iter().filter(|s| s.step_id == StepId::ApothecaryMultiple).collect();
        assert_eq!(apos.len(), 2);
    }
}
