/// BB2020 Select Gaze Target step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2020.SelectGazeTarget`.
use ffb_model::enums::ApothecaryMode;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

pub struct SelectGazeTarget;

impl SelectGazeTarget {
    pub fn new() -> Self { Self }

    pub fn build_sequence() -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        let fl = labels::END_SELECTING;

        // 1 SELECT_GAZE_TARGET [SELECT]
        seq.add_labelled(StepId::SelectGazeTarget, labels::SELECT, vec![
            StepParameter::GotoLabelOnEnd(fl.into()),
        ]);

        // ACTIVATION BLOCK (with GotoLabel, fl=END_SELECTING, BloodLust with failure label)
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

        // FOUL_APPEARANCE → END_SELECTING
        seq.add(StepId::FoulAppearance, vec![StepParameter::GotoLabelOnFailure(fl.into())]);
        // JUMP_UP → END_SELECTING
        seq.add(StepId::JumpUp, vec![StepParameter::GotoLabelOnFailure(fl.into())]);
        // STAND_UP → END_SELECTING
        seq.add(StepId::StandUp, vec![StepParameter::GotoLabelOnFailure(fl.into())]);
        // SELECT_GAZE_TARGET_END [END_SELECTING]
        seq.add_labelled(StepId::SelectGazeTargetEnd, labels::END_SELECTING, vec![]);

        seq.build()
    }
}

impl Default for SelectGazeTarget {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_gaze_target_is_labelled_select() {
        let steps = SelectGazeTarget::build_sequence();
        let s = steps.iter().find(|s| s.step_id == StepId::SelectGazeTarget).unwrap();
        assert_eq!(s.label.as_deref(), Some(labels::SELECT));
    }

    #[test]
    fn select_gaze_target_end_is_labelled_end_selecting() {
        let steps = SelectGazeTarget::build_sequence();
        let s = steps.iter().find(|s| s.step_id == StepId::SelectGazeTargetEnd).unwrap();
        assert_eq!(s.label.as_deref(), Some(labels::END_SELECTING));
    }

    #[test]
    fn select_gaze_target_has_activation_block() {
        let steps = SelectGazeTarget::build_sequence();
        assert!(steps.iter().any(|s| s.step_id == StepId::InitActivation));
    }

    #[test]
    fn select_gaze_target_has_foul_appearance() {
        let steps = SelectGazeTarget::build_sequence();
        assert!(steps.iter().any(|s| s.step_id == StepId::FoulAppearance));
    }

    #[test]
    fn select_gaze_target_blood_lust_has_failure_label() {
        let steps = SelectGazeTarget::build_sequence();
        let bl = steps.iter().find(|s| s.step_id == StepId::BloodLust).unwrap();
        let has = bl.params.iter().any(|p| matches!(p, StepParameter::GotoLabelOnFailure(_)));
        assert!(has);
    }

    #[test]
    fn select_gaze_target_has_no_dump_off() {
        let steps = SelectGazeTarget::build_sequence();
        assert!(!steps.iter().any(|s| s.step_id == StepId::DumpOff));
    }
}
