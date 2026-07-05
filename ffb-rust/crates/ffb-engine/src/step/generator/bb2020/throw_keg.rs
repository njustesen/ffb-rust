/// BB2020 Throw Keg step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2020.ThrowKeg`.
use ffb_model::enums::ApothecaryMode;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

#[derive(Debug, Clone, Default)]
pub struct ThrowKegParams {
    pub player_id: Option<String>,
}

pub struct ThrowKeg;

impl ThrowKeg {
    pub fn new() -> Self { Self }

    pub fn build_sequence(params: &ThrowKegParams) -> Vec<SequenceStep> {
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

        // THROW_KEG (TargetPlayerId)
        let mut keg_params = vec![];
        if let Some(ref id) = params.player_id {
            keg_params.push(StepParameter::TargetPlayerId(Some(id.clone())));
        }
        seq.add(StepId::ThrowKeg, keg_params);
        // HANDLE_DROP_PLAYER_CONTEXT
        seq.add(StepId::HandleDropPlayerContext, vec![]);
        // APOTHECARY (defender)
        seq.add(StepId::Apothecary, vec![StepParameter::ApothecaryMode(ApothecaryMode::Defender)]);
        // CATCH_SCATTER_THROW_IN
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        // END_THROW_KEG [END]
        seq.add_labelled(StepId::EndThrowKeg, labels::END, vec![]);

        seq.build()
    }
}

impl Default for ThrowKeg {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn throw_keg_has_activation_block() {
        let steps = ThrowKeg::build_sequence(&ThrowKegParams::default());
        assert!(steps.iter().any(|s| s.step_id == StepId::InitActivation));
    }

    #[test]
    fn throw_keg_ends_with_end_throw_keg_labelled_end() {
        let steps = ThrowKeg::build_sequence(&ThrowKegParams::default());
        let last = steps.last().unwrap();
        assert_eq!(last.step_id, StepId::EndThrowKeg);
        assert_eq!(last.label.as_deref(), Some(labels::END));
    }

    #[test]
    fn throw_keg_bone_head_has_no_label() {
        let steps = ThrowKeg::build_sequence(&ThrowKegParams::default());
        let bh = steps.iter().find(|s| s.step_id == StepId::BoneHead).unwrap();
        assert!(bh.label.is_none());
    }

    #[test]
    fn throw_keg_has_no_steady_footing() {
        let steps = ThrowKeg::build_sequence(&ThrowKegParams::default());
        assert!(!steps.iter().any(|s| s.step_id == StepId::SteadyFooting));
    }

    #[test]
    fn player_id_passed_when_some() {
        let params = ThrowKegParams { player_id: Some("target_player".into()) };
        let steps = ThrowKeg::build_sequence(&params);
        let keg = steps.iter().find(|s| s.step_id == StepId::ThrowKeg).unwrap();
        let has = keg.params.iter().any(|p| matches!(p, StepParameter::TargetPlayerId(Some(id)) if id == "target_player"));
        assert!(has);
    }

    #[test]
    fn player_id_absent_when_none() {
        let steps = ThrowKeg::build_sequence(&ThrowKegParams::default());
        let keg = steps.iter().find(|s| s.step_id == StepId::ThrowKeg).unwrap();
        assert!(keg.params.is_empty());
    }
}
