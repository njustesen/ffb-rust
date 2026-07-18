/// BB2025 Raiding Party step sequence (single-step).
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2025.RaidingParty`.
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};
use super::activation_sequence_builder::ActivationSequenceBuilder;

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

        // [ACTIVATION(END)]
        ActivationSequenceBuilder::new()
            .with_failure_label(labels::END)
            .add_to(&mut seq);

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
    fn raiding_party_last_step_labelled_end() {
        let steps = RaidingParty::build_sequence(&RaidingPartyParams {
            failure_label: "fail".into(),
            success_label: "ok".into(),
        });
        let last = steps.last().unwrap();
        assert_eq!(last.step_id, StepId::RaidingParty);
        assert_eq!(last.label.as_deref(), Some(labels::END));
    }

    #[test]
    fn activation_sub_sequence_precedes_raiding_party() {
        // Java pushSequence: ActivationSequenceBuilder.create()...addTo(sequence) before RAIDING_PARTY.
        let steps = RaidingParty::build_sequence(&RaidingPartyParams::default());
        assert_eq!(steps.len(), 14);
        assert_eq!(steps[0].step_id, StepId::InitActivation);
    }

    #[test]
    fn failure_label_wired() {
        let steps = RaidingParty::build_sequence(&RaidingPartyParams {
            failure_label: "FAIL_LABEL".into(), success_label: "OK".into()
        });
        let last = steps.last().unwrap();
        assert!(last.params.iter().any(|p| matches!(p, StepParameter::GotoLabelOnFailure(l) if l == "FAIL_LABEL")));
    }

    #[test]
    fn success_label_wired() {
        let steps = RaidingParty::build_sequence(&RaidingPartyParams {
            failure_label: "F".into(), success_label: "SUCCESS_LABEL".into()
        });
        let last = steps.last().unwrap();
        assert!(last.params.iter().any(|p| matches!(p, StepParameter::GotoLabelOnSuccess(l) if l == "SUCCESS_LABEL")));
    }

    #[test]
    fn params_with_fields_set() {
        let p = RaidingPartyParams { failure_label: "fail".into(), success_label: "ok".into() };
        assert_eq!(p.failure_label, "fail");
        assert_eq!(p.success_label, "ok");
    }

    #[test]
    fn params_clone() {
        let p = RaidingPartyParams { failure_label: "f".into(), success_label: "s".into() };
        let q = p.clone();
        assert_eq!(q.failure_label, "f");
        assert_eq!(q.success_label, "s");
    }
}
