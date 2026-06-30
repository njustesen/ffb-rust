use ffb_model::enums::TurnMode;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// Initialisation step in the end-game sequence (BB2020).
///
/// Key differences from BB2025:
/// - When a team concedes illegally, publishes TOUCHDOWNS (conceder.score + 1) and TEAM_ID
///   so that StepAssignTouchdowns can award them to winning-team players.
/// - adjustScore: winner = winner.score + conceder.score + 1; conceder = 0.
///   (BB2025 uses max(winner.score, 2) instead.)
///
/// 1:1 translation of com.fumbbl.ffb.server.step.bb2020.end.StepInitEndGame.
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
            StepParameter::GotoLabelOnEnd(label) => { self.goto_label_on_end = label.clone(); true }
            StepParameter::AdminMode(v) => { self.admin_mode = *v; true }
            _ => false,
        }
    }
}

impl StepInitEndGame {
    fn execute_step(&self, game: &mut Game) -> StepOutcome {
        if game.is_finished() {
            return StepOutcome::goto(&self.goto_label_on_end);
        }

        let mut outcome = StepOutcome::next();

        if game.game_result.home.conceded {
            let touchdowns = game.game_result.home.score + 1;
            let team_id = game.team_away.id.clone();
            outcome = outcome
                .publish(StepParameter::Touchdowns(touchdowns))
                .publish(StepParameter::TeamId(team_id));
            // Java: adjustScore(away, home): away.score += home.score + 1; home.score = 0
            let conceder_score = game.game_result.home.score;
            game.game_result.away.score += conceder_score + 1;
            game.game_result.home.score = 0;
        } else if game.game_result.away.conceded {
            let touchdowns = game.game_result.away.score + 1;
            let team_id = game.team_home.id.clone();
            outcome = outcome
                .publish(StepParameter::Touchdowns(touchdowns))
                .publish(StepParameter::TeamId(team_id));
            let conceder_score = game.game_result.away.score;
            game.game_result.home.score += conceder_score + 1;
            game.game_result.away.score = 0;
        }

        game.turn_mode = TurnMode::EndGame;
        game.concession_possible = false;
        game.admin_mode = self.admin_mode;

        outcome
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::{GameStatus, Rules};

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

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

    #[test]
    fn finished_game_goes_to_label() {
        let mut game = make_game();
        game.status = GameStatus::Finished;
        let mut step = StepInitEndGame::new("my_label".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("my_label"));
    }

    #[test]
    fn home_conceded_adjusts_scores_bb2020_formula() {
        // BB2020: away.score = away.score + home.score + 1; home.score = 0
        let mut game = make_game();
        game.game_result.home.conceded = true;
        game.game_result.home.score = 2;
        game.game_result.away.score = 1;
        let mut step = StepInitEndGame::new("end".into());
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.game_result.home.score, 0);
        assert_eq!(game.game_result.away.score, 4); // 1 + 2 + 1
    }

    #[test]
    fn away_conceded_adjusts_scores_bb2020_formula() {
        let mut game = make_game();
        game.game_result.away.conceded = true;
        game.game_result.away.score = 1;
        game.game_result.home.score = 2;
        let mut step = StepInitEndGame::new("end".into());
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.game_result.away.score, 0);
        assert_eq!(game.game_result.home.score, 4); // 2 + 1 + 1
    }

    #[test]
    fn home_conceded_publishes_touchdowns_and_team_id() {
        let mut game = make_game();
        game.game_result.home.conceded = true;
        game.game_result.home.score = 2;
        let mut step = StepInitEndGame::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        let has_touchdowns = out.published.iter().any(|p| matches!(p, StepParameter::Touchdowns(3)));
        let has_team_id = out.published.iter().any(|p| matches!(p, StepParameter::TeamId(_)));
        assert!(has_touchdowns, "should publish Touchdowns = home.score + 1 = 3");
        assert!(has_team_id, "should publish TeamId for away team");
    }

    #[test]
    fn away_conceded_publishes_touchdowns_and_team_id() {
        let mut game = make_game();
        game.game_result.away.conceded = true;
        game.game_result.away.score = 0;
        let mut step = StepInitEndGame::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        let has_touchdowns = out.published.iter().any(|p| matches!(p, StepParameter::Touchdowns(1)));
        let has_team_id = out.published.iter().any(|p| matches!(p, StepParameter::TeamId(_)));
        assert!(has_touchdowns, "should publish Touchdowns = away.score + 1 = 1");
        assert!(has_team_id);
    }

    #[test]
    fn no_concession_publishes_nothing() {
        let mut game = make_game();
        let mut step = StepInitEndGame::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.is_empty());
    }

    #[test]
    fn admin_mode_applied_to_game() {
        let mut game = make_game();
        let mut step = StepInitEndGame::new("end".into());
        step.admin_mode = true;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.admin_mode);
    }

    #[test]
    fn set_parameter_goto_label_accepted() {
        let mut step = StepInitEndGame::new("init".into());
        assert!(step.set_parameter(&StepParameter::GotoLabelOnEnd("custom".into())));
        assert_eq!(step.goto_label_on_end, "custom");
    }

    #[test]
    fn set_parameter_admin_mode_accepted() {
        let mut step = StepInitEndGame::new("end".into());
        assert!(step.set_parameter(&StepParameter::AdminMode(true)));
        assert!(step.admin_mode);
    }

    #[test]
    fn unrecognised_parameter_returns_false() {
        let mut step = StepInitEndGame::new("end".into());
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }
}
