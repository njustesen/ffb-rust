/// BB2025 Catch of the Day step sequence (single-step).
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2025.CatchOfTheDay`.
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};
use super::activation_sequence_builder::ActivationSequenceBuilder;

#[derive(Debug, Clone, Default)]
pub struct CatchOfTheDayParams {
    pub failure_label: String,
}

pub struct CatchOfTheDay;

impl CatchOfTheDay {
    pub fn new() -> Self { Self }

    pub fn build_sequence(params: &CatchOfTheDayParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();

        // 1-13 [ACTIVATION(END)]
        ActivationSequenceBuilder::new()
            .with_failure_label(labels::END)
            .add_to(&mut seq);

        // 14 CATCH_OF_THE_DAY [END]
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
    fn catch_of_the_day_last_step_labelled_end() {
        let steps = CatchOfTheDay::build_sequence(&CatchOfTheDayParams { failure_label: "X".into() });
        let last = steps.last().unwrap();
        assert_eq!(last.step_id, StepId::CatchOfTheDay);
        assert_eq!(last.label.as_deref(), Some(labels::END));
    }

    #[test]
    fn failure_label_in_params() {
        let steps = CatchOfTheDay::build_sequence(&CatchOfTheDayParams { failure_label: "lbl".into() });
        let last = steps.last().unwrap();
        let has = last.params.iter().any(|p| {
            matches!(p, StepParameter::GotoLabelOnFailure(l) if l == "lbl")
        });
        assert!(has);
    }

    #[test]
    fn step_id_is_catch_of_the_day() {
        let steps = CatchOfTheDay::build_sequence(&CatchOfTheDayParams::default());
        assert_eq!(steps.last().unwrap().step_id, StepId::CatchOfTheDay);
    }

    #[test]
    fn activation_sub_sequence_precedes_catch_of_the_day() {
        // Java pushSequence: ActivationSequenceBuilder.create()...addTo(sequence) before CATCH_OF_THE_DAY.
        let steps = CatchOfTheDay::build_sequence(&CatchOfTheDayParams::default());
        assert_eq!(steps.len(), 14);
        assert_eq!(steps[0].step_id, StepId::InitActivation);
    }

    #[test]
    fn params_with_fields_set() {
        let p = CatchOfTheDayParams { failure_label: "myLabel".into() };
        assert_eq!(p.failure_label, "myLabel");
    }

    #[test]
    fn params_clone() {
        let p = CatchOfTheDayParams { failure_label: "lbl".into() };
        let q = p.clone();
        assert_eq!(q.failure_label, "lbl");
    }
}
