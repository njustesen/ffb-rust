/// BB2020 Treacherous step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2020.Treacherous`.
use ffb_model::enums::ApothecaryMode;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

#[derive(Debug, Clone, Default)]
pub struct TreacherousParams {
    pub failure_label: String,
}

pub struct Treacherous;

impl Treacherous {
    pub fn new() -> Self { Self }

    pub fn build_sequence(params: &TreacherousParams) -> Vec<SequenceStep> {
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

        // JUMP_UP → END
        seq.add(StepId::JumpUp, vec![StepParameter::GotoLabelOnFailure(fl.into())]);
        // STAND_UP → END
        seq.add(StepId::StandUp, vec![StepParameter::GotoLabelOnFailure(fl.into())]);
        // TREACHEROUS [END] (→ failure_label)
        seq.add_labelled(StepId::Treacherous, labels::END, vec![
            StepParameter::GotoLabelOnFailure(params.failure_label.clone()),
        ]);
        // HANDLE_DROP_PLAYER_CONTEXT
        seq.add(StepId::HandleDropPlayerContext, vec![]);
        // APOTHECARY (defender)
        seq.add(StepId::Apothecary, vec![StepParameter::ApothecaryMode(ApothecaryMode::Defender)]);

        seq.build()
    }
}

impl Default for Treacherous {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn treacherous_has_activation_block() {
        let steps = Treacherous::build_sequence(&TreacherousParams::default());
        assert!(steps.iter().any(|s| s.step_id == StepId::InitActivation));
    }

    #[test]
    fn treacherous_is_labelled_end() {
        let steps = Treacherous::build_sequence(&TreacherousParams { failure_label: "X".into() });
        let t = steps.iter().find(|s| s.step_id == StepId::Treacherous).unwrap();
        assert_eq!(t.label.as_deref(), Some(labels::END));
    }

    #[test]
    fn treacherous_bone_head_has_no_label() {
        let steps = Treacherous::build_sequence(&TreacherousParams::default());
        let bh = steps.iter().find(|s| s.step_id == StepId::BoneHead).unwrap();
        assert!(bh.label.is_none());
    }

    #[test]
    fn treacherous_ends_with_apothecary_defender() {
        let steps = Treacherous::build_sequence(&TreacherousParams::default());
        let last = steps.last().unwrap();
        assert_eq!(last.step_id, StepId::Apothecary);
    }
}
