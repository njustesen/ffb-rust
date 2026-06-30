/// BB2020 Select Blitz Target step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2020.SelectBlitzTarget`.
use ffb_model::enums::ApothecaryMode;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

pub struct SelectBlitzTarget;

impl SelectBlitzTarget {
    pub fn new() -> Self { Self }

    pub fn build_sequence() -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        let fl = labels::END_BLITZING;

        // 1 SELECT_BLITZ_TARGET [SELECT]
        seq.add_labelled(StepId::SelectBlitzTarget, labels::SELECT, vec![
            StepParameter::GotoLabelOnEnd(fl.into()),
        ]);

        // ACTIVATION BLOCK (with GotoLabel, fl=END_BLITZING, BloodLust with failure label)
        let fl_s: String = fl.into();
        seq.add(StepId::InitActivation, vec![]);
        seq.add(StepId::AnimalSavagery, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);
        seq.add(StepId::HandleDropPlayerContext, vec![]);
        seq.add(StepId::PlaceBall, vec![]);
        seq.add(StepId::Apothecary, vec![StepParameter::ApothecaryMode(ApothecaryMode::AnimalSavagery)]);
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        seq.add(StepId::GotoLabel, vec![
            StepParameter::GotoLabel(labels::NEXT.into()),
            StepParameter::AlternateGotoLabel(fl_s.clone()),
        ]);
        seq.add_labelled(StepId::BoneHead, labels::NEXT, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);
        seq.add(StepId::ReallyStupid, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);
        seq.add(StepId::TakeRoot, vec![]);
        seq.add(StepId::UnchannelledFury, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);
        seq.add(StepId::BloodLust, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);

        // FOUL_APPEARANCE → END_BLITZING
        seq.add(StepId::FoulAppearance, vec![StepParameter::GotoLabelOnFailure(fl.into())]);
        // DUMP_OFF
        seq.add(StepId::DumpOff, vec![]);
        // JUMP_UP → END_BLITZING
        seq.add(StepId::JumpUp, vec![StepParameter::GotoLabelOnFailure(fl.into())]);
        // STAND_UP → END_BLITZING
        seq.add(StepId::StandUp, vec![StepParameter::GotoLabelOnFailure(fl.into())]);
        // SELECT_BLITZ_TARGET_END [END_BLITZING]
        seq.add_labelled(StepId::SelectBlitzTargetEnd, labels::END_BLITZING, vec![]);

        seq.build()
    }
}

impl Default for SelectBlitzTarget {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_blitz_target_is_labelled_select() {
        let steps = SelectBlitzTarget::build_sequence();
        let s = steps.iter().find(|s| s.step_id == StepId::SelectBlitzTarget).unwrap();
        assert_eq!(s.label.as_deref(), Some(labels::SELECT));
    }

    #[test]
    fn select_blitz_target_end_is_labelled_end_blitzing() {
        let steps = SelectBlitzTarget::build_sequence();
        let s = steps.iter().find(|s| s.step_id == StepId::SelectBlitzTargetEnd).unwrap();
        assert_eq!(s.label.as_deref(), Some(labels::END_BLITZING));
    }

    #[test]
    fn select_blitz_target_has_activation_block() {
        let steps = SelectBlitzTarget::build_sequence();
        assert!(steps.iter().any(|s| s.step_id == StepId::InitActivation));
    }

    #[test]
    fn select_blitz_target_has_dump_off() {
        let steps = SelectBlitzTarget::build_sequence();
        assert!(steps.iter().any(|s| s.step_id == StepId::DumpOff));
    }
}
