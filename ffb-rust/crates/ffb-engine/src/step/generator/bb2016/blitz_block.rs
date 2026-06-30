/// BB2016 blitz-block step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2016.BlitzBlock`.
/// Sequence identical to BB2016 Block.
use crate::step::generator::bb2016::block::{Block, BlockParams};
use crate::step::generator::sequence::SequenceStep;

/// Parameters for the BB2016 BlitzBlock sequence — same as Block.
pub type BlitzBlockParams = BlockParams;

pub struct BlitzBlock;

impl BlitzBlock {
    pub fn new() -> Self { Self }

    pub fn build_sequence(params: &BlitzBlockParams) -> Vec<SequenceStep> {
        Block::build_sequence(params)
    }
}

impl Default for BlitzBlock {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::StepId;
    use crate::step::generator::sequence::labels;

    #[test]
    fn blitz_block_sequence_starts_with_init_blocking() {
        let steps = BlitzBlock::build_sequence(&BlitzBlockParams::default());
        assert_eq!(steps[0].step_id, StepId::InitBlocking);
    }

    #[test]
    fn blitz_block_sequence_ends_with_end_blocking() {
        let steps = BlitzBlock::build_sequence(&BlitzBlockParams::default());
        assert_eq!(steps.last().unwrap().step_id, StepId::EndBlocking);
    }

    #[test]
    fn blitz_block_omits_foul_appearance_when_frenzy() {
        let steps = BlitzBlock::build_sequence(&BlitzBlockParams { frenzy_block: true, ..Default::default() });
        assert!(!steps.iter().any(|s| s.step_id == StepId::FoulAppearance));
    }

    #[test]
    fn blitz_block_has_apothecary_defender_label() {
        let steps = BlitzBlock::build_sequence(&BlitzBlockParams::default());
        assert!(steps.iter().any(|s| s.label.as_deref() == Some(labels::APOTHECARY_DEFENDER)));
    }
}
