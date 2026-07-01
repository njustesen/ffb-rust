/// 1:1 translation of `com.fumbbl.ffb.server.step.game.start.StepInitStartGame`.
///
/// Java: gates the start-game sequence on both coaches signalling ready and (in FUMBBL
/// server mode) a FUMBBL game-state record being created. In standalone mode the step
/// transitions immediately once both teams are present (which is always true in our engine).
///
/// Rust: only standalone mode is supported. `start()` → `execute_step()` → `leave_step()`
/// immediately, setting `GameStatus::Active` and returning `NextStep`.
use ffb_model::enums::GameStatus;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepCommandStatus, StepId, StepOutcome};

pub struct StepInitStartGame {
    /// Java: `fFumbblGameCreated` — true once the FUMBBL API acknowledged the game.
    /// Unused in standalone mode; retained for serialization parity.
    fumbbl_game_created: bool,
    /// Java: `fStartedHome` — true once the home coach sent `CLIENT_START_GAME`.
    started_home: bool,
    /// Java: `fStartedAway` — true once the away coach sent `CLIENT_START_GAME`.
    started_away: bool,
}

impl StepInitStartGame {
    pub fn new() -> Self {
        Self {
            fumbbl_game_created: false,
            started_home: false,
            started_away: false,
        }
    }
}

impl Default for StepInitStartGame {
    fn default() -> Self { Self::new() }
}

impl Step for StepInitStartGame {
    fn id(&self) -> StepId { StepId::InitStartGame }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        let command_status = match action {
            Action::StartGame { home } => {
                if *home {
                    self.started_home = true;
                } else {
                    self.started_away = true;
                }
                StepCommandStatus::ExecuteStep
            }
            _ => StepCommandStatus::UnhandledCommand,
        };
        if command_status == StepCommandStatus::ExecuteStep {
            self.execute_step(game)
        } else {
            StepOutcome::cont()
        }
    }
}

impl StepInitStartGame {
    /// Java: `executeStep()` — proceeds to `leave_step()` in standalone mode.
    ///
    /// Java standalone: calls `leaveStep()` directly once `game.getStarted() != null`.
    /// In our engine, teams are always loaded at game start, so we proceed immediately.
    fn execute_step(&self, game: &mut Game) -> StepOutcome {
        self.leave_step(game)
    }

    /// Java: `leaveStep()` — sets the game active and advances to the next step.
    fn leave_step(&self, game: &mut Game) -> StepOutcome {
        game.status = GameStatus::Active;
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
    fn id_is_init_start_game() {
        let step = StepInitStartGame::new();
        assert_eq!(step.id(), StepId::InitStartGame);
    }

    #[test]
    fn default_fields_are_false() {
        let step = StepInitStartGame::new();
        assert!(!step.fumbbl_game_created);
        assert!(!step.started_home);
        assert!(!step.started_away);
    }

    #[test]
    fn start_sets_game_active_and_returns_next_step() {
        let mut step = StepInitStartGame::new();
        let mut game = make_game();
        let mut rng = make_rng();
        let outcome = step.start(&mut game, &mut rng);
        assert_eq!(outcome.action, StepAction::NextStep);
        assert_eq!(game.status, GameStatus::Active);
    }

    #[test]
    fn start_initial_status_is_starting() {
        let game = make_game();
        assert_eq!(game.status, GameStatus::Starting);
    }

    #[test]
    fn handle_command_start_game_home_sets_started_home() {
        let mut step = StepInitStartGame::new();
        let mut game = make_game();
        let mut rng = make_rng();
        step.handle_command(&Action::StartGame { home: true }, &mut game, &mut rng);
        assert!(step.started_home);
        assert!(!step.started_away);
    }

    #[test]
    fn handle_command_start_game_away_sets_started_away() {
        let mut step = StepInitStartGame::new();
        let mut game = make_game();
        let mut rng = make_rng();
        step.handle_command(&Action::StartGame { home: false }, &mut game, &mut rng);
        assert!(!step.started_home);
        assert!(step.started_away);
    }

    #[test]
    fn handle_command_start_game_also_activates_game() {
        let mut step = StepInitStartGame::new();
        let mut game = make_game();
        let mut rng = make_rng();
        let outcome = step.handle_command(&Action::StartGame { home: true }, &mut game, &mut rng);
        assert_eq!(outcome.action, StepAction::NextStep);
        assert_eq!(game.status, GameStatus::Active);
    }

    #[test]
    fn handle_command_unknown_action_returns_continue() {
        let mut step = StepInitStartGame::new();
        let mut game = make_game();
        let mut rng = make_rng();
        let outcome = step.handle_command(&Action::EndTurn, &mut game, &mut rng);
        assert_eq!(outcome.action, StepAction::Continue);
    }

    #[test]
    fn default_creates_same_as_new() {
        let s = StepInitStartGame::default();
        assert_eq!(s.id(), StepId::InitStartGame);
    }
}
