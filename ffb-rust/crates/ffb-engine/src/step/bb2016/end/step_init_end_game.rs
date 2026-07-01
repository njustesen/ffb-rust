/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.end.StepInitEndGame`.
///
/// Initialises the end-of-game sequence:
/// - If the game is already finished → goto label.
/// - Adjusts scores for illegal concession: winning team gets at least 1 point more.
/// - Sets TurnMode::EndGame, disables concession.
/// - `handlePoisonedPlayers` deferred (SeriousInjuryFactory not yet ported).
use ffb_model::enums::TurnMode;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepInitEndGame` (bb2016/end).
pub struct StepInitEndGame {
    /// Java: `fGotoLabelOnEnd` — init param (mandatory).
    goto_label_on_end: String,
    /// Java: `fAdminMode` — optional init param.
    admin_mode: bool,
}

impl StepInitEndGame {
    pub fn new() -> Self {
        Self { goto_label_on_end: String::new(), admin_mode: false }
    }

    fn execute_step(&self, game: &mut Game) -> StepOutcome {
        if game.is_finished() {
            return StepOutcome::goto(&self.goto_label_on_end);
        }

        // BB2016: ensure winner has at least 1 more point than the conceding team.
        if game.game_result.home.conceded {
            let score_diff_away = game.game_result.away.score - game.game_result.home.score;
            if score_diff_away <= 0 {
                game.game_result.away.score += score_diff_away.abs() + 1;
            }
        }
        if game.game_result.away.conceded {
            let score_diff_home = game.game_result.home.score - game.game_result.away.score;
            if score_diff_home <= 0 {
                game.game_result.home.score += score_diff_home.abs() + 1;
            }
        }

        game.turn_mode = TurnMode::EndGame;
        game.concession_possible = false;
        game.admin_mode = self.admin_mode;
        // DEFERRED(CardEffect): handlePoisonedPlayers requires SeriousInjuryFactory + CardEffect::POISONED

        StepOutcome::next()
    }
}

impl Default for StepInitEndGame {
    fn default() -> Self { Self::new() }
}

impl Step for StepInitEndGame {
    fn id(&self) -> StepId { StepId::InitEndGame }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(label) => { self.goto_label_on_end = label.clone(); true }
            StepParameter::AdminMode(v)           => { self.admin_mode = *v; true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::{GameStatus, Rules};

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    #[test]
    fn id_is_init_end_game() {
        assert_eq!(StepInitEndGame::new().id(), StepId::InitEndGame);
    }

    #[test]
    fn normal_game_sets_end_game_mode() {
        let mut game = make_game();
        game.concession_possible = true;
        let mut step = StepInitEndGame::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
        assert_eq!(game.turn_mode, TurnMode::EndGame);
        assert!(!game.concession_possible);
    }

    #[test]
    fn finished_game_goes_to_label() {
        let mut game = make_game();
        game.status = GameStatus::Finished;
        let mut step = StepInitEndGame::new();
        step.set_parameter(&StepParameter::GotoLabelOnEnd("end_label".into()));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::GotoLabel));
        assert_eq!(out.goto_label.as_deref(), Some("end_label"));
    }

    #[test]
    fn home_conceded_losing_bumps_away_score() {
        // home=3, away=1 → scoreDiffAway = 1-3 = -2 ≤ 0 → away += 2+1 = 3 → away=4
        let mut game = make_game();
        game.game_result.home.conceded = true;
        game.game_result.home.score = 3;
        game.game_result.away.score = 1;
        let mut step = StepInitEndGame::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.game_result.away.score, 4);
        assert_eq!(game.game_result.home.score, 3); // home score untouched
    }

    #[test]
    fn home_conceded_already_ahead_no_change() {
        // away=3, home=1 → scoreDiffAway = 2 > 0 → no bump
        let mut game = make_game();
        game.game_result.home.conceded = true;
        game.game_result.home.score = 1;
        game.game_result.away.score = 3;
        let mut step = StepInitEndGame::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.game_result.away.score, 3);
    }

    #[test]
    fn set_parameter_goto_label_and_admin_mode() {
        let mut step = StepInitEndGame::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnEnd("lbl".into())));
        assert_eq!(step.goto_label_on_end, "lbl");
        assert!(step.set_parameter(&StepParameter::AdminMode(true)));
        assert!(step.admin_mode);
    }
}
