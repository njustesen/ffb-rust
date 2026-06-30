// TODO: full implementation. Stub placeholder for TRANSLATION_TRACKER.md.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepId, StepOutcome};

pub struct StepTouchback;

impl StepTouchback {
    pub fn new() -> Self { Self }
}

impl Default for StepTouchback {
    fn default() -> Self { Self::new() }
}

impl Step for StepTouchback {
    fn id(&self) -> StepId { StepId::Touchback }
    fn start(&mut self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome { StepOutcome::next() }
    fn handle_command(&mut self, _action: &Action, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome { StepOutcome::next() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::Rules;
    use ffb_model::model::game::Game;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn step_id_is_touchback() {
        let step = StepTouchback::new();
        assert_eq!(step.id(), StepId::Touchback);
    }

    #[test]
    fn start_returns_next_step() {
        let mut step = StepTouchback::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn handle_command_returns_next_step() {
        let mut step = StepTouchback::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.handle_command(&Action::ConfirmSetup, &mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn default_works() {
        let _step = StepTouchback::default();
    }
}
