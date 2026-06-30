/// Builds the Quick Bite step sequence (BB2020/BB2025).
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.mixed.QuickBite`.
use ffb_model::enums::ApothecaryMode;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep};

pub struct QuickBite;

impl QuickBite {
    pub fn new() -> Self { Self }

    /// Build the mixed quick-bite step sequence (Java `pushSequence`).
    pub fn build_sequence() -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        // 1 QUICK_BITE
        seq.add(StepId::QuickBite, vec![]);
        // 2 HANDLE_DROP_PLAYER_CONTEXT
        seq.add(StepId::HandleDropPlayerContext, vec![]);
        // 3 APOTHECARY (QUICK_BITE)
        seq.add(StepId::Apothecary, vec![StepParameter::ApothecaryMode(ApothecaryMode::QuickBite)]);
        seq.build()
    }
}

impl Default for QuickBite {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quick_bite_has_three_steps() {
        let steps = QuickBite::build_sequence();
        assert_eq!(steps.len(), 3);
    }

    #[test]
    fn quick_bite_starts_with_quick_bite() {
        let steps = QuickBite::build_sequence();
        assert_eq!(steps[0].step_id, StepId::QuickBite);
    }

    #[test]
    fn quick_bite_ends_with_apothecary() {
        let steps = QuickBite::build_sequence();
        assert_eq!(steps.last().unwrap().step_id, StepId::Apothecary);
    }
}
