/// Builds the end-turn step sequence (BB2016/BB2020).
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.mixed.EndTurn`.
use crate::step::framework::StepId;
use crate::step::generator::sequence::{Sequence, SequenceStep};

pub struct EndTurn;

impl EndTurn {
    pub fn new() -> Self { Self }

    /// Build the mixed end-turn step sequence (Java `pushSequence`).
    pub fn build_sequence() -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        // 1 END_TURN (new sequence may be inserted at this point at runtime)
        seq.add(StepId::EndTurn, vec![]);
        seq.build()
    }
}

impl Default for EndTurn {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn end_turn_has_one_step() {
        let steps = EndTurn::build_sequence();
        assert_eq!(steps.len(), 1);
    }

    #[test]
    fn end_turn_starts_with_end_turn() {
        let steps = EndTurn::build_sequence();
        assert_eq!(steps[0].step_id, StepId::EndTurn);
    }

    #[test]
    fn end_turn_step_has_no_label() {
        let steps = EndTurn::build_sequence();
        assert!(steps[0].label.is_none());
    }
}
