/// BB2020 Raiding Party step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2020.RaidingParty`.
use ffb_model::enums::ApothecaryMode;
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
        let fl = labels::END;

        // ACTIVATION BLOCK (BalefulHex-style)
        let fl_s: String = fl.into();
        seq.add(StepId::InitActivation, vec![]);
        seq.add(StepId::AnimalSavagery, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);
        seq.add(StepId::HandleDropPlayerContext, vec![]);
        seq.add(StepId::PlaceBall, vec![]);
        seq.add(StepId::Apothecary, vec![StepParameter::ApothecaryMode(ApothecaryMode::AnimalSavagery)]);
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        seq.add(StepId::BoneHead, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);  // no label
        seq.add(StepId::ReallyStupid, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);
        seq.add(StepId::TakeRoot, vec![]);
        seq.add(StepId::UnchannelledFury, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);
        seq.add(StepId::BloodLust, vec![]);

        // RAIDING_PARTY [END] (→ failure_label, → success_label)
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
    fn raiding_party_has_activation_block() {
        let steps = RaidingParty::build_sequence(&RaidingPartyParams::default());
        assert!(steps.iter().any(|s| s.step_id == StepId::InitActivation));
    }

    #[test]
    fn raiding_party_ends_labelled_end() {
        let steps = RaidingParty::build_sequence(&RaidingPartyParams {
            failure_label: "fail".into(),
            success_label: "ok".into(),
        });
        let last = steps.last().unwrap();
        assert_eq!(last.step_id, StepId::RaidingParty);
        assert_eq!(last.label.as_deref(), Some(labels::END));
    }

    #[test]
    fn raiding_party_bone_head_has_no_label() {
        let steps = RaidingParty::build_sequence(&RaidingPartyParams::default());
        let bh = steps.iter().find(|s| s.step_id == StepId::BoneHead).unwrap();
        assert!(bh.label.is_none());
    }

    #[test]
    fn raiding_party_blood_lust_has_no_failure_label() {
        let steps = RaidingParty::build_sequence(&RaidingPartyParams::default());
        let bl = steps.iter().find(|s| s.step_id == StepId::BloodLust).unwrap();
        assert!(!bl.params.iter().any(|p| matches!(p, StepParameter::GotoLabelOnFailure(_))));
    }
}
