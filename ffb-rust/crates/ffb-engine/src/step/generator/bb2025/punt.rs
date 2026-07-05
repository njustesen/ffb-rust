/// BB2025 punt step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2025.Punt`.
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

pub struct Punt;

impl Punt {
    pub fn new() -> Self { Self }

    pub fn build_sequence() -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        // 1 INIT_PUNT
        seq.add(StepId::InitPunt, vec![
            StepParameter::GotoLabelOnEnd(labels::END.into()),
        ]);
        // 2 PUNT_DIRECTION
        seq.add(StepId::PuntDirection, vec![
            StepParameter::GotoLabelOnEnd(labels::SCATTER_BALL.into()),
        ]);
        // 3 PUNT_DISTANCE
        seq.add(StepId::PuntDistance, vec![]);
        // 4 CATCH_SCATTER_THROW_IN [SCATTER_BALL]
        seq.add_labelled(StepId::CatchScatterThrowIn, labels::SCATTER_BALL, vec![]);
        // 5 END_PUNT [END]
        seq.add_labelled(StepId::EndPunt, labels::END, vec![]);
        seq.build()
    }
}

impl Default for Punt {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn punt_has_5_steps() {
        let steps = Punt::build_sequence();
        assert_eq!(steps.len(), 5);
    }

    #[test]
    fn punt_ends_with_end_punt_labelled_end() {
        let steps = Punt::build_sequence();
        let last = steps.last().unwrap();
        assert_eq!(last.step_id, StepId::EndPunt);
        assert_eq!(last.label.as_deref(), Some(labels::END));
    }

    #[test]
    fn punt_catch_scatter_is_labelled_scatter_ball() {
        let steps = Punt::build_sequence();
        let cst = steps.iter().find(|s| s.step_id == StepId::CatchScatterThrowIn).unwrap();
        assert_eq!(cst.label.as_deref(), Some(labels::SCATTER_BALL));
    }

    #[test]
    fn punt_starts_with_init_punt() {
        let steps = Punt::build_sequence();
        assert_eq!(steps[0].step_id, StepId::InitPunt);
    }

    #[test]
    fn punt_has_punt_direction_and_punt_distance() {
        let steps = Punt::build_sequence();
        assert!(steps.iter().any(|s| s.step_id == StepId::PuntDirection));
        assert!(steps.iter().any(|s| s.step_id == StepId::PuntDistance));
    }
}
