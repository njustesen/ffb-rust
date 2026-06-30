/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.shared.StepConsumeParameter`.
///
/// Absorbs specific parameter variants so they do not propagate further down the stack.
/// The set of variants to absorb is configured at construction time via PARAMETERS_TO_CONSUME
/// init parameters (each carrying a list of `std::mem::Discriminant<StepParameter>` values).
use std::collections::HashSet;
use std::mem::discriminant;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepConsumeParameter` (mixed/shared, BB2020 + BB2025).
pub struct StepConsumeParameter {
    /// Java: `parameterToConsume` — discriminants of StepParameter variants to absorb.
    parameters_to_consume: HashSet<std::mem::Discriminant<StepParameter>>,
}

impl StepConsumeParameter {
    pub fn new() -> Self {
        Self { parameters_to_consume: HashSet::new() }
    }
}

impl Default for StepConsumeParameter {
    fn default() -> Self { Self::new() }
}

impl Step for StepConsumeParameter {
    fn id(&self) -> StepId { StepId::ConsumeParameter }

    fn start(&mut self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        StepOutcome::next()
    }

    fn handle_command(&mut self, _action: &Action, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        StepOutcome::next()
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::ParametersToConsume(keys) => {
                for k in keys { self.parameters_to_consume.insert(k.clone()); }
                true
            }
            other => self.parameters_to_consume.contains(&discriminant(other)),
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use ffb_model::enums::Rules;
    use ffb_model::model::game::Game;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn id_is_consume_parameter() {
        assert_eq!(StepConsumeParameter::new().id(), StepId::ConsumeParameter);
    }

    #[test]
    fn start_returns_next_step() {
        let mut step = StepConsumeParameter::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert!(matches!(outcome.action, crate::step::framework::StepAction::NextStep));
    }

    #[test]
    fn consumes_registered_parameter() {
        let mut step = StepConsumeParameter::new();
        let marker = StepParameter::EndTurn(false);
        step.set_parameter(&StepParameter::ParametersToConsume(vec![
            std::mem::discriminant(&marker),
        ]));
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
    }

    #[test]
    fn does_not_consume_unregistered_parameter() {
        let mut step = StepConsumeParameter::new();
        let marker = StepParameter::EndTurn(false);
        step.set_parameter(&StepParameter::ParametersToConsume(vec![
            std::mem::discriminant(&marker),
        ]));
        // EndPlayerAction not registered → not consumed
        assert!(!step.set_parameter(&StepParameter::EndPlayerAction(false)));
    }

    #[test]
    fn parameters_to_consume_itself_is_consumed() {
        let mut step = StepConsumeParameter::new();
        let consumed = step.set_parameter(&StepParameter::ParametersToConsume(vec![]));
        assert!(consumed);
    }

    #[test]
    fn multiple_parameter_kinds_registered() {
        let mut step = StepConsumeParameter::new();
        step.set_parameter(&StepParameter::ParametersToConsume(vec![
            std::mem::discriminant(&StepParameter::EndTurn(false)),
            std::mem::discriminant(&StepParameter::EndPlayerAction(false)),
        ]));
        assert!(step.set_parameter(&StepParameter::EndTurn(false)));
        assert!(step.set_parameter(&StepParameter::EndPlayerAction(false)));
        assert!(!step.set_parameter(&StepParameter::AdminMode(false)));
    }
}
