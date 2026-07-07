/// BB2020 Baleful Hex step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2020.BalefulHex`.
use ffb_model::enums::ApothecaryMode;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

#[derive(Debug, Clone, Default)]
pub struct BalefulHexParams {
    pub failure_label: String,
}

pub struct BalefulHex;

impl BalefulHex {
    pub fn new() -> Self { Self }

    pub fn build_sequence(params: &BalefulHexParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        let fl = labels::END;

        // ACTIVATION BLOCK (BalefulHex-style: no GotoLabel, BoneHead without label, BloodLust no failure)
        let fl_s: String = fl.into();
        seq.add(StepId::InitActivation, vec![]);
        seq.add(StepId::AnimalSavagery, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);
        seq.add(StepId::HandleDropPlayerContext, vec![]);
        seq.add(StepId::PlaceBall, vec![]);
        seq.add(StepId::Apothecary, vec![StepParameter::ApothecaryMode(ApothecaryMode::AnimalSavagery)]);
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        // NO GotoLabel step
        seq.add(StepId::BoneHead, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);  // no label
        seq.add(StepId::ReallyStupid, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);
        seq.add(StepId::TakeRoot, vec![]);
        seq.add(StepId::UnchannelledFury, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);
        seq.add(StepId::BloodLust, vec![]);  // no failure label

        // BALEFUL_HEX [END] (→ failure_label)
        seq.add_labelled(StepId::BalefulHex, labels::END, vec![
            StepParameter::GotoLabelOnFailure(params.failure_label.clone()),
        ]);

        seq.build()
    }
}

impl Default for BalefulHex {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baleful_hex_has_activation_block() {
        let steps = BalefulHex::build_sequence(&BalefulHexParams { failure_label: "X".into() });
        assert!(steps.iter().any(|s| s.step_id == StepId::InitActivation));
    }

    #[test]
    fn baleful_hex_ends_with_baleful_hex_labelled_end() {
        let steps = BalefulHex::build_sequence(&BalefulHexParams { failure_label: "X".into() });
        let last = steps.last().unwrap();
        assert_eq!(last.step_id, StepId::BalefulHex);
        assert_eq!(last.label.as_deref(), Some(labels::END));
    }

    #[test]
    fn baleful_hex_bone_head_has_no_label() {
        let steps = BalefulHex::build_sequence(&BalefulHexParams::default());
        let bh = steps.iter().find(|s| s.step_id == StepId::BoneHead).unwrap();
        assert!(bh.label.is_none());
    }

    #[test]
    fn baleful_hex_blood_lust_has_no_failure_label() {
        let steps = BalefulHex::build_sequence(&BalefulHexParams::default());
        let bl = steps.iter().find(|s| s.step_id == StepId::BloodLust).unwrap();
        assert!(!bl.params.iter().any(|p| matches!(p, StepParameter::GotoLabelOnFailure(_))));
    }
    #[test]
    fn build_sequence_is_nonempty() {
        assert!(!BalefulHex::build_sequence(&Default::default()).is_empty());
    }

}
