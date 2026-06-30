/// BB2025 end-turn step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2025.EndTurn`.
use ffb_model::enums::ApothecaryMode;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep};

/// Parameters for the EndTurn sequence — mirrors Java `EndTurn.SequenceParams`.
#[derive(Debug, Clone, Default)]
pub struct EndTurnParams {
    pub check_forgo: bool,
}

pub struct EndTurn;

impl EndTurn {
    pub fn new() -> Self { Self }

    /// Build the end-turn step sequence (Java `pushSequence`).
    pub fn build_sequence(params: &EndTurnParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();

        // 1 FORGONE_STALLING
        seq.add(StepId::ForgoneStalling, vec![
            StepParameter::CheckForgo(params.check_forgo),
        ]);
        // 2 STEADY_FOOTING (HIT_PLAYER)
        seq.add(StepId::SteadyFooting, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::HitPlayer),
        ]);
        // 3 PLACE_BALL
        seq.add(StepId::PlaceBall, vec![]);
        // 4 APOTHECARY (HIT_PLAYER)
        seq.add(StepId::Apothecary, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::HitPlayer),
        ]);
        // 5 CATCH_SCATTER_THROW_IN
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        // 6 END_TURN
        seq.add(StepId::EndTurn, vec![]);

        seq.build()
    }
}

impl Default for EndTurn {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn end_turn_sequence_has_six_steps() {
        let steps = EndTurn::build_sequence(&EndTurnParams::default());
        assert_eq!(steps.len(), 6);
    }

    #[test]
    fn end_turn_sequence_starts_with_forgone_stalling() {
        let steps = EndTurn::build_sequence(&EndTurnParams::default());
        assert_eq!(steps[0].step_id, StepId::ForgoneStalling);
    }

    #[test]
    fn end_turn_sequence_ends_with_end_turn() {
        let steps = EndTurn::build_sequence(&EndTurnParams::default());
        assert_eq!(steps.last().unwrap().step_id, StepId::EndTurn);
    }

    #[test]
    fn end_turn_check_forgo_param_is_forwarded() {
        let steps = EndTurn::build_sequence(&EndTurnParams { check_forgo: true });
        let forgone = &steps[0];
        assert!(forgone.params.iter().any(|p| matches!(p, StepParameter::CheckForgo(true))));
    }
}
