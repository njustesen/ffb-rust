/// Builds the Spiked Ball apothecary step sequence (all editions).
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.common.SpikedBallApo`.
use ffb_model::enums::ApothecaryMode;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep};

pub struct SpikedBallApo;

impl SpikedBallApo {
    pub fn new() -> Self { Self }

    /// Build the spiked-ball apothecary step sequence (Java `pushSequence`).
    pub fn build_sequence() -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        // 1 APOTHECARY (CATCHER)
        seq.add(StepId::Apothecary, vec![StepParameter::ApothecaryMode(ApothecaryMode::Catcher)]);
        seq.build()
    }
}

impl Default for SpikedBallApo {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spiked_ball_apo_has_one_step() {
        let steps = SpikedBallApo::build_sequence();
        assert_eq!(steps.len(), 1);
    }

    #[test]
    fn spiked_ball_apo_starts_with_apothecary() {
        let steps = SpikedBallApo::build_sequence();
        assert_eq!(steps[0].step_id, StepId::Apothecary);
    }

    #[test]
    fn spiked_ball_apo_step_has_no_label() {
        let steps = SpikedBallApo::build_sequence();
        assert!(steps[0].label.is_none());
    }
}
