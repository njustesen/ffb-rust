/// BB2020 Catch of the Day step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2020.CatchOfTheDay`.
use ffb_model::enums::ApothecaryMode;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

#[derive(Debug, Clone, Default)]
pub struct CatchOfTheDayParams {
    pub failure_label: String,
}

pub struct CatchOfTheDay;

impl CatchOfTheDay {
    pub fn new() -> Self { Self }

    pub fn build_sequence(params: &CatchOfTheDayParams) -> Vec<SequenceStep> {
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

        // CATCH_OF_THE_DAY [END] (→ failure_label)
        seq.add_labelled(StepId::CatchOfTheDay, labels::END, vec![
            StepParameter::GotoLabelOnFailure(params.failure_label.clone()),
        ]);

        seq.build()
    }
}

impl Default for CatchOfTheDay {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catch_of_the_day_has_activation_block() {
        let steps = CatchOfTheDay::build_sequence(&CatchOfTheDayParams::default());
        assert!(steps.iter().any(|s| s.step_id == StepId::InitActivation));
    }

    #[test]
    fn catch_of_the_day_ends_labelled_end() {
        let steps = CatchOfTheDay::build_sequence(&CatchOfTheDayParams { failure_label: "X".into() });
        let last = steps.last().unwrap();
        assert_eq!(last.step_id, StepId::CatchOfTheDay);
        assert_eq!(last.label.as_deref(), Some(labels::END));
    }

    #[test]
    fn catch_of_the_day_bone_head_has_no_label() {
        let steps = CatchOfTheDay::build_sequence(&CatchOfTheDayParams::default());
        let bh = steps.iter().find(|s| s.step_id == StepId::BoneHead).unwrap();
        assert!(bh.label.is_none());
    }

    #[test]
    fn catch_of_the_day_blood_lust_has_no_failure_label() {
        let steps = CatchOfTheDay::build_sequence(&CatchOfTheDayParams::default());
        let bl = steps.iter().find(|s| s.step_id == StepId::BloodLust).unwrap();
        assert!(!bl.params.iter().any(|p| matches!(p, StepParameter::GotoLabelOnFailure(_))));
    }

    #[test]
    fn failure_label_passed_to_catch_of_the_day_step() {
        let steps = CatchOfTheDay::build_sequence(&CatchOfTheDayParams { failure_label: "myLabel".into() });
        let cotd = steps.iter().find(|s| s.step_id == StepId::CatchOfTheDay).unwrap();
        let has = cotd.params.iter().any(|p| matches!(p, StepParameter::GotoLabelOnFailure(l) if l == "myLabel"));
        assert!(has);
    }

    #[test]
    fn catch_of_the_day_step_count() {
        let steps = CatchOfTheDay::build_sequence(&CatchOfTheDayParams::default());
        assert_eq!(steps.len(), 12);
    }
}
