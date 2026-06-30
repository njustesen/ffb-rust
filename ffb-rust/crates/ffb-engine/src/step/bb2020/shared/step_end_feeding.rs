use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// Final step of the feed sequence. Consumes EndPlayerAction/EndTurn.
/// 1:1 translation of com.fumbbl.ffb.server.step.bb2020.shared.StepEndFeeding.
///
/// BB2020 differs from BB2025: no CheckForgo parameter.
/// Full logic requires sequence generators.
pub struct StepEndFeeding {
    pub end_player_action: bool,
    pub end_turn: bool,
}

impl StepEndFeeding {
    pub fn new() -> Self {
        Self { end_player_action: false, end_turn: false }
    }
}

impl Default for StepEndFeeding {
    fn default() -> Self { Self::new() }
}

impl Step for StepEndFeeding {
    fn id(&self) -> StepId { StepId::EndFeeding }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::EndPlayerAction(v) => { self.end_player_action = *v; true }
            StepParameter::EndTurn(v) => { self.end_turn = *v; true }
            _ => false,
        }
    }
}

impl StepEndFeeding {
    fn execute_step(&mut self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // TODO(generators): full BB2020 StepEndFeeding logic requires sequence generators.
        // BB2020 end_turn in PASS_BLOCK mode → use EndTurn generator;
        // otherwise more complex inducement/select sequence. Stubbed for now.
        StepOutcome::next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2020)
    }

    #[test]
    fn set_parameter_end_turn_accepted() {
        let mut step = StepEndFeeding::new();
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.end_turn);
    }

    #[test]
    fn set_parameter_end_player_action_accepted() {
        let mut step = StepEndFeeding::new();
        assert!(step.set_parameter(&StepParameter::EndPlayerAction(true)));
        assert!(step.end_player_action);
    }

    #[test]
    fn start_returns_next() {
        let mut game = make_game();
        let mut step = StepEndFeeding::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn no_check_forgo_parameter() {
        // BB2020 StepEndFeeding does not accept CheckForgo (unlike BB2025).
        let mut step = StepEndFeeding::new();
        assert!(!step.set_parameter(&StepParameter::CheckForgo(true)));
    }
}
