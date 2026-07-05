/// BB2025 Throw a Rock kickoff result step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2025.ThrowARock`.
use ffb_model::enums::ApothecaryMode;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep};

#[derive(Debug, Clone, Default)]
pub struct ThrowARockParams {
    pub home_team: bool,
}

pub struct ThrowARock;

impl ThrowARock {
    pub fn new() -> Self { Self }

    pub fn build_sequence(params: &ThrowARockParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        // 1 THROW_A_ROCK
        seq.add(StepId::ThrowARock, vec![
            StepParameter::HomeTeam(params.home_team),
        ]);
        // 2 STEADY_FOOTING (DEFENDER)
        seq.add(StepId::SteadyFooting, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::Defender),
        ]);
        // 3 HANDLE_DROP_PLAYER_CONTEXT
        seq.add(StepId::HandleDropPlayerContext, vec![]);
        // 4 PLACE_BALL
        seq.add(StepId::PlaceBall, vec![]);
        // 5 APOTHECARY (DEFENDER)
        seq.add(StepId::Apothecary, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::Defender),
        ]);
        // 6 CATCH_SCATTER_THROW_IN
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        seq.build()
    }
}

impl Default for ThrowARock {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn throw_a_rock_has_6_steps() {
        let steps = ThrowARock::build_sequence(&ThrowARockParams::default());
        assert_eq!(steps.len(), 6);
    }

    #[test]
    fn throw_a_rock_starts_with_throw_a_rock() {
        let steps = ThrowARock::build_sequence(&ThrowARockParams::default());
        assert_eq!(steps[0].step_id, StepId::ThrowARock);
    }

    #[test]
    fn home_team_param_passed_to_first_step() {
        let steps = ThrowARock::build_sequence(&ThrowARockParams { home_team: true });
        assert!(steps[0].params.iter().any(|p| matches!(p, StepParameter::HomeTeam(true))));
    }

    #[test]
    fn away_team_param_passed_to_first_step() {
        let steps = ThrowARock::build_sequence(&ThrowARockParams { home_team: false });
        assert!(steps[0].params.iter().any(|p| matches!(p, StepParameter::HomeTeam(false))));
    }

    #[test]
    fn contains_catch_scatter_throw_in_as_last() {
        let steps = ThrowARock::build_sequence(&ThrowARockParams::default());
        assert_eq!(steps.last().unwrap().step_id, StepId::CatchScatterThrowIn);
    }
}
