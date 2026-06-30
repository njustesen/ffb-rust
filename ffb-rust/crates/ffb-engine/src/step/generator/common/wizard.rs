/// Builds the Wizard spell step sequence (all editions).
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.common.Wizard`.
use crate::step::framework::StepId;
use crate::step::generator::sequence::{Sequence, SequenceStep};

pub struct Wizard;

impl Wizard {
    pub fn new() -> Self { Self }

    /// Build the wizard step sequence (Java `pushSequence`).
    pub fn build_sequence() -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        // 1 WIZARD (multiple specialEffect sequences may be inserted after at runtime)
        seq.add(StepId::Wizard, vec![]);
        // 2 CATCH_SCATTER_THROW_IN
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        seq.build()
    }
}

impl Default for Wizard {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wizard_has_two_steps() {
        let steps = Wizard::build_sequence();
        assert_eq!(steps.len(), 2);
    }

    #[test]
    fn wizard_starts_with_wizard() {
        let steps = Wizard::build_sequence();
        assert_eq!(steps[0].step_id, StepId::Wizard);
    }

    #[test]
    fn wizard_ends_with_catch_scatter_throw_in() {
        let steps = Wizard::build_sequence();
        assert_eq!(steps.last().unwrap().step_id, StepId::CatchScatterThrowIn);
    }
}
