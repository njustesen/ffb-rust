use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// TODO: full logic.
pub struct StepInitKickoff;

impl StepInitKickoff {
    pub fn new() -> Self { Self }
}

impl Default for StepInitKickoff {
    fn default() -> Self { Self::new() }
}

impl Step for StepInitKickoff {
    fn id(&self) -> StepId { StepId::InitKickoff }
    fn start(&mut self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome { StepOutcome::next() }
    fn handle_command(&mut self, _action: &Action, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome { StepOutcome::next() }
    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::model::game::Game;
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    #[test]
    fn id_is_init_kickoff() {
        assert_eq!(StepInitKickoff::new().id(), StepId::InitKickoff);
    }

    #[test]
    fn start_returns_next_step() {
        let mut step = StepInitKickoff::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert!(matches!(out.action, StepAction::NextStep));
    }

    #[test]
    fn handle_command_returns_next_step() {
        let mut step = StepInitKickoff::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let action = crate::action::Action::Acknowledge;
        let out = step.handle_command(&action, &mut game, &mut rng);
        assert!(matches!(out.action, StepAction::NextStep));
    }

    #[test]
    fn set_parameter_returns_false() {
        let mut step = StepInitKickoff::new();
        let param = StepParameter::EndTurn(false);
        assert!(!step.set_parameter(&param));
    }
}
