use ffb_model::enums::TurnMode;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// Initialisation step in the end-game sequence. Sets TurnMode::EndGame, disables concession,
/// and adjusts the score when a team conceded illegally (winner ≥ 2; loser = 0).
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.end.StepInitEndGame`.
pub struct StepInitEndGame {
    pub goto_label_on_end: String,
    pub admin_mode: bool,
}

impl StepInitEndGame {
    pub fn new(goto_label_on_end: String) -> Self {
        Self { goto_label_on_end, admin_mode: false }
    }
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
            StepParameter::GotoLabelOnEnd(label) => {
                self.goto_label_on_end = label.clone();
                true
            }
            StepParameter::AdminMode(v) => {
                self.admin_mode = *v;
                true
            }
            _ => false,
        }
    }
}

impl StepInitEndGame {
    fn execute_step(&self, game: &mut Game) -> StepOutcome {
        if game.is_finished() {
            return StepOutcome::goto(&self.goto_label_on_end);
        }
        // Adjust scores for illegal concession: winner gets at least 2, loser gets 0.
        // Java: adjustScore(winnerResult, concedingResult) sets concedingResult.setScore(0)
        // unconditionally and winnerResult.setScore(Math.max(score, 2)).
        if game.game_result.home.conceded {
            let away_score = game.game_result.away.score;
            game.game_result.away.score = away_score.max(2);
            game.game_result.home.score = 0; // always 0 — Java: concedingResult.setScore(0)
        } else if game.game_result.away.conceded {
            let home_score = game.game_result.home.score;
            game.game_result.home.score = home_score.max(2);
            game.game_result.away.score = 0; // always 0 — Java: concedingResult.setScore(0)
        }
        game.turn_mode = TurnMode::EndGame;
        game.concession_possible = false;
        game.admin_mode = self.admin_mode;
        StepOutcome::next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::{GameStatus, Rules};

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    /// Normal (no concession): sets TurnMode::EndGame, disables concession, returns next.
    #[test]
    fn normal_game_sets_end_game_mode() {
        let mut game = make_game();
        game.concession_possible = true;
        let mut step = StepInitEndGame::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.turn_mode, TurnMode::EndGame);
        assert!(!game.concession_possible);
    }

    /// Already-finished game: should goto the label immediately without changing turn mode.
    #[test]
    fn finished_game_goes_to_label() {
        let mut game = make_game();
        game.status = GameStatus::Finished;
        let mut step = StepInitEndGame::new("my_label".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("my_label"));
    }

    /// Home conceded (illegally): conceding score is always set to 0 even if already 0.
    #[test]
    fn home_concession_sets_home_score_to_zero() {
        let mut game = make_game();
        game.game_result.home.conceded = true;
        game.game_result.home.score = 3;
        game.game_result.away.score = 1;
        let mut step = StepInitEndGame::new("end".into());
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.game_result.home.score, 0);
    }

    /// Home conceded: winner (away) score is raised to at least 2.
    #[test]
    fn home_concession_raises_away_score_to_at_least_2() {
        let mut game = make_game();
        game.game_result.home.conceded = true;
        game.game_result.home.score = 2;
        game.game_result.away.score = 0; // below minimum, must be bumped
        let mut step = StepInitEndGame::new("end".into());
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.game_result.away.score, 2);
        assert_eq!(game.game_result.home.score, 0);
    }

    /// Away conceded: away score → 0, home score → max(existing, 2).
    #[test]
    fn away_concession_adjusts_scores_correctly() {
        let mut game = make_game();
        game.game_result.away.conceded = true;
        game.game_result.away.score = 1;
        game.game_result.home.score = 3; // already above 2, should stay at 3
        let mut step = StepInitEndGame::new("end".into());
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.game_result.away.score, 0);
        assert_eq!(game.game_result.home.score, 3);
    }

    /// set_parameter: GotoLabelOnEnd and AdminMode are accepted.
    #[test]
    fn set_parameter_goto_label() {
        let mut step = StepInitEndGame::new("init".into());
        assert!(step.set_parameter(&StepParameter::GotoLabelOnEnd("custom".into())));
        assert_eq!(step.goto_label_on_end, "custom");
    }

    #[test]
    fn set_parameter_admin_mode() {
        let mut step = StepInitEndGame::new("end".into());
        assert!(step.set_parameter(&StepParameter::AdminMode(true)));
        assert!(step.admin_mode);
    }

    #[test]
    fn admin_mode_is_applied_to_game() {
        let mut game = make_game();
        let mut step = StepInitEndGame::new("end".into());
        step.admin_mode = true;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.admin_mode);
    }
}
