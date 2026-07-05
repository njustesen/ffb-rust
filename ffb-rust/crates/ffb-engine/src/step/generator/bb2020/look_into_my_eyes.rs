/// BB2020 Look Into My Eyes step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2020.LookIntoMyEyes`.
use ffb_model::enums::ApothecaryMode;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

#[derive(Debug, Clone, Default)]
pub struct LookIntoMyEyesParams {
    pub push_select: bool,
    pub goto_on_end: String,
}

pub struct LookIntoMyEyes;

impl LookIntoMyEyes {
    pub fn new() -> Self { Self }

    pub fn build_sequence(params: &LookIntoMyEyesParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        let fl = labels::END;

        // ACTIVATION BLOCK (with GotoLabel, BloodLust no failure)
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
        seq.add(StepId::BloodLust, vec![]);

        // INIT_LOOK_INTO_MY_EYES
        seq.add(StepId::InitLookIntoMyEyes, vec![]);
        // FOUL_APPEARANCE → END
        seq.add(StepId::FoulAppearance, vec![StepParameter::GotoLabelOnFailure(fl.into())]);
        // LOOK_INTO_MY_EYES [END]
        seq.add_labelled(StepId::LookIntoMyEyes, labels::END, vec![
            StepParameter::PushSelect(params.push_select),
            StepParameter::GotoLabelOnEnd(params.goto_on_end.clone()),
        ]);

        seq.build()
    }
}

impl Default for LookIntoMyEyes {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn look_into_my_eyes_has_activation_block() {
        let steps = LookIntoMyEyes::build_sequence(&LookIntoMyEyesParams::default());
        assert!(steps.iter().any(|s| s.step_id == StepId::InitActivation));
    }

    #[test]
    fn look_into_my_eyes_step_is_labelled_end() {
        let steps = LookIntoMyEyes::build_sequence(&LookIntoMyEyesParams::default());
        let s = steps.iter().find(|s| s.step_id == StepId::LookIntoMyEyes).unwrap();
        assert_eq!(s.label.as_deref(), Some(labels::END));
    }

    #[test]
    fn look_into_my_eyes_has_foul_appearance() {
        let steps = LookIntoMyEyes::build_sequence(&LookIntoMyEyesParams::default());
        assert!(steps.iter().any(|s| s.step_id == StepId::FoulAppearance));
    }

    #[test]
    fn look_into_my_eyes_blood_lust_has_no_failure_label() {
        let steps = LookIntoMyEyes::build_sequence(&LookIntoMyEyesParams::default());
        let bl = steps.iter().find(|s| s.step_id == StepId::BloodLust).unwrap();
        assert!(!bl.params.iter().any(|p| matches!(p, StepParameter::GotoLabelOnFailure(_))));
    }

    #[test]
    fn push_select_param_passed_to_look_into_my_eyes_step() {
        let params = LookIntoMyEyesParams { push_select: true, goto_on_end: "end".into() };
        let steps = LookIntoMyEyes::build_sequence(&params);
        let s = steps.iter().find(|s| s.step_id == StepId::LookIntoMyEyes).unwrap();
        let has = s.params.iter().any(|p| matches!(p, StepParameter::PushSelect(true)));
        assert!(has);
    }

    #[test]
    fn goto_on_end_param_passed_to_look_into_my_eyes_step() {
        let params = LookIntoMyEyesParams { push_select: false, goto_on_end: "myEnd".into() };
        let steps = LookIntoMyEyes::build_sequence(&params);
        let s = steps.iter().find(|s| s.step_id == StepId::LookIntoMyEyes).unwrap();
        let has = s.params.iter().any(|p| matches!(p, StepParameter::GotoLabelOnEnd(l) if l == "myEnd"));
        assert!(has);
    }
}
