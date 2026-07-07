use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// 1:1 translation of com.fumbbl.ffb.server.step.StepNextStep.
/// Advances to the next step unconditionally.
pub struct StepNextStep;

impl StepNextStep {
    pub fn new() -> Self { Self }
}

impl Default for StepNextStep {
    fn default() -> Self { Self::new() }
}

impl Step for StepNextStep {
    fn id(&self) -> StepId { StepId::NextStep }
    fn start(&mut self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome { StepOutcome::next() }
    fn handle_command(&mut self, _action: &Action, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome { StepOutcome::next() }
    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;
    use crate::action::Action;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn id_is_next_step() { assert_eq!(StepNextStep::new().id(), StepId::NextStep); }

    #[test]
    fn set_parameter_returns_false() { assert!(!StepNextStep::new().set_parameter(&StepParameter::EndTurn(true))); }

    #[test]
    fn start_returns_next_step_action() {
        let mut step = StepNextStep::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert_eq!(outcome.action, StepAction::NextStep);
    }

    #[test]
    fn handle_command_returns_next_step_action() {
        let mut step = StepNextStep::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let outcome = step.handle_command(&Action::Acknowledge, &mut game, &mut rng);
        assert_eq!(outcome.action, StepAction::NextStep);
    }
}
