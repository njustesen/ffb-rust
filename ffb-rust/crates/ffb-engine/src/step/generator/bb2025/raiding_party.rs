/// BB2025 Raiding Party step sequence (single-step).
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2025.RaidingParty`.
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

#[derive(Debug, Clone, Default)]
pub struct RaidingPartyParams {
    pub failure_label: String,
    pub success_label: String,
}

pub struct RaidingParty;

impl RaidingParty {
    pub fn new() -> Self { Self }

    pub fn build_sequence(params: &RaidingPartyParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        seq.add_labelled(StepId::RaidingParty, labels::END, vec![
            StepParameter::GotoLabelOnFailure(params.failure_label.clone()),
            StepParameter::GotoLabelOnSuccess(params.success_label.clone()),
        ]);
        seq.build()
    }
}

impl Default for RaidingParty {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raiding_party_single_step_labelled_end() {
        let steps = RaidingParty::build_sequence(&RaidingPartyParams {
            failure_label: "fail".into(),
            success_label: "ok".into(),
        });
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].step_id, StepId::RaidingParty);
        assert_eq!(steps[0].label.as_deref(), Some(labels::END));
    }
}
