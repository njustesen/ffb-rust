/// 1:1 translation of `com.fumbbl.ffb.server.step.game.end.StepEndGame`.
///
/// Java: final step in the end-game sequence. Sets the game finished timestamp,
/// transitions to `GameStatus.FINISHED`, shows the game-statistics dialog, and
/// (in non-test standalone mode) saves a replay file.
///
/// Rust: sets `game.status = GameStatus::Finished` and returns `NextStep`.
/// Replay saving and dialog display are DEFERRED (no file I/O in engine).
use ffb_model::enums::GameStatus;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepId, StepOutcome};

pub struct StepEndGame;

impl StepEndGame {
    pub fn new() -> Self { Self }
}

impl Default for StepEndGame {
    fn default() -> Self { Self::new() }
}

impl Step for StepEndGame {
    fn id(&self) -> StepId { StepId::EndGame }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        StepOutcome::cont()
    }
}

impl StepEndGame {
    /// Java: `executeStep()` — sets the game finished and transitions to FINISHED status.
    fn execute_step(&self, game: &mut Game) -> StepOutcome {
        game.status = GameStatus::Finished;
        // DEFERRED(dialog): show GameStatistics dialog (requires dialog infra)
        // DEFERRED(replay): save replay file (requires file I/O infra)
        StepOutcome::next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn make_rng() -> GameRng {
        GameRng::new(42)
    }

    #[test]
    fn id_is_end_game() {
        let step = StepEndGame::new();
        assert_eq!(step.id(), StepId::EndGame);
    }

    #[test]
    fn start_sets_game_finished_and_returns_next_step() {
        let mut step = StepEndGame::new();
        let mut game = make_game();
        let mut rng = make_rng();
        let outcome = step.start(&mut game, &mut rng);
        assert_eq!(outcome.action, StepAction::NextStep);
        assert_eq!(game.status, GameStatus::Finished);
    }

    #[test]
    fn start_initial_status_is_starting() {
        let game = make_game();
        assert_eq!(game.status, GameStatus::Starting);
    }

    #[test]
    fn handle_command_returns_continue() {
        let mut step = StepEndGame::new();
        let mut game = make_game();
        let mut rng = make_rng();
        let outcome = step.handle_command(&Action::EndTurn, &mut game, &mut rng);
        assert_eq!(outcome.action, StepAction::Continue);
    }

    #[test]
    fn default_creates_same_as_new() {
        let s = StepEndGame::default();
        assert_eq!(s.id(), StepId::EndGame);
    }
}
