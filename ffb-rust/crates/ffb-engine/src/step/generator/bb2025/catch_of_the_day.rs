/// BB2025 Catch of the Day step sequence (single-step).
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2025.CatchOfTheDay`.
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
    fn catch_of_the_day_single_step_labelled_end() {
        let steps = CatchOfTheDay::build_sequence(&CatchOfTheDayParams { failure_label: "X".into() });
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].step_id, StepId::CatchOfTheDay);
        assert_eq!(steps[0].label.as_deref(), Some(labels::END));
    }
}
