/// Builds the Riotous Rookies kickoff result step sequence (all editions).
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.common.RiotousRookies`.
/// Java pushes a single StepRiotousRookies directly (not via Sequence builder).
use crate::step::framework::StepId;
use crate::step::generator::sequence::{Sequence, SequenceStep};

pub struct RiotousRookies;

impl RiotousRookies {
    pub fn new() -> Self { Self }

    /// Build the riotous-rookies step sequence (Java `pushSequence`).
    pub fn build_sequence() -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        // 1 RIOTOUS_ROOKIES (single step — Java pushes StepRiotousRookies directly)
        seq.add(StepId::RiotousRookies, vec![]);
        seq.build()
    }
}

impl Default for RiotousRookies {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn riotous_rookies_has_one_step() {
        let steps = RiotousRookies::build_sequence();
        assert_eq!(steps.len(), 1);
    }

    #[test]
    fn riotous_rookies_starts_with_riotous_rookies() {
        let steps = RiotousRookies::build_sequence();
        assert_eq!(steps[0].step_id, StepId::RiotousRookies);
    }

    #[test]
    fn riotous_rookies_step_has_no_label() {
        let steps = RiotousRookies::build_sequence();
        assert!(steps[0].label.is_none());
    }
}
