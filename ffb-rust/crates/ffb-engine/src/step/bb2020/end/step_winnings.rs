use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::report::mixed::report_winnings::ReportWinnings;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::StepId;

/// Calculates and sets each team's winnings at the end of the game (BB2020).
/// Formula: score × 10,000 gp per point + attendance (fan_factor split).
/// On illegal concession the winning team receives all attendance.
/// No stalled-team bonus (BB2020 rule difference vs BB2025).
/// 1:1 translation of com.fumbbl.ffb.server.step.bb2020.end.StepWinnings.
pub struct StepWinnings;

impl Step for StepWinnings {
    fn id(&self) -> StepId { StepId::Winnings }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }
}

impl StepWinnings {
    fn execute_step(&self, game: &mut Game) -> StepOutcome {
        let attendance = (game.game_result.home.fan_factor + game.game_result.away.fan_factor) as f64;
        let mut home_winnings = game.game_result.home.score as f64;
        let mut away_winnings = game.game_result.away.score as f64;

        if game.game_result.home.conceded && !game.conceded_legally {
            away_winnings += attendance;
        } else if game.game_result.away.conceded && !game.conceded_legally {
            home_winnings += attendance;
        } else {
            home_winnings += attendance / 2.0;
            away_winnings += attendance / 2.0;
        }

        game.game_result.home.winnings = (home_winnings * 10_000.0) as i32;
        game.game_result.away.winnings = (away_winnings * 10_000.0) as i32;

        // Java: getResult().addReport(new ReportWinnings((int) homeWinnings, (int) awayWinnings))
        game.report_list.add(ReportWinnings::new(
            game.game_result.home.winnings,
            game.game_result.away.winnings,
        ));

        StepOutcome::next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::Rules;
    use ffb_model::report::report_id::ReportId;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    #[test]
    fn normal_game_splits_attendance_evenly() {
        let mut game = make_game();
        let mut step = StepWinnings;
        game.game_result.home.score = 2;
        game.game_result.away.score = 1;
        game.game_result.home.fan_factor = 5;
        game.game_result.away.fan_factor = 5;
        // attendance = 10, home = 2+5 = 7, away = 1+5 = 6
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.game_result.home.winnings, 70_000);
        assert_eq!(game.game_result.away.winnings, 60_000);
    }

    #[test]
    fn returns_next_step() {
        let mut game = make_game();
        let mut step = StepWinnings;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn illegal_home_concession_gives_all_attendance_to_away() {
        let mut game = make_game();
        let mut step = StepWinnings;
        game.game_result.home.conceded = true;
        game.conceded_legally = false;
        game.game_result.home.score = 0;
        game.game_result.away.score = 2;
        game.game_result.home.fan_factor = 3;
        game.game_result.away.fan_factor = 4;
        // away = 2 + 7 = 9 → 90_000, home = 0
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.game_result.home.winnings, 0);
        assert_eq!(game.game_result.away.winnings, 90_000);
    }

    #[test]
    fn illegal_away_concession_gives_all_attendance_to_home() {
        let mut game = make_game();
        let mut step = StepWinnings;
        game.game_result.away.conceded = true;
        game.conceded_legally = false;
        game.game_result.home.score = 2;
        game.game_result.away.score = 0;
        game.game_result.home.fan_factor = 4;
        game.game_result.away.fan_factor = 3;
        // home = 2 + 7 = 9 → 90_000, away = 0
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.game_result.home.winnings, 90_000);
        assert_eq!(game.game_result.away.winnings, 0);
    }

    #[test]
    fn legal_concession_splits_normally() {
        let mut game = make_game();
        let mut step = StepWinnings;
        game.game_result.home.conceded = true;
        game.conceded_legally = true;
        game.game_result.home.score = 1;
        game.game_result.away.score = 2;
        game.game_result.home.fan_factor = 2;
        game.game_result.away.fan_factor = 2;
        // attendance = 4, home = 1+2 = 3 → 30_000, away = 2+2 = 4 → 40_000
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.game_result.home.winnings, 30_000);
        assert_eq!(game.game_result.away.winnings, 40_000);
    }

    #[test]
    fn no_stalled_bonus_in_bb2020() {
        // BB2020 does NOT add +1 for non-stalled teams (differs from BB2025)
        let mut game = make_game();
        let mut step = StepWinnings;
        game.game_result.home.score = 1;
        game.game_result.away.score = 1;
        game.game_result.home.stalled = false;
        game.game_result.away.stalled = false;
        // attendance = 0, each team gets score * 10_000 = 10_000
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.game_result.home.winnings, 10_000);
        assert_eq!(game.game_result.away.winnings, 10_000);
    }

    #[test]
    fn zero_scores_zero_fan_factor_gives_zero_winnings() {
        let mut game = make_game();
        let mut step = StepWinnings;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.game_result.home.winnings, 0);
        assert_eq!(game.game_result.away.winnings, 0);
    }

    #[test]
    fn adds_winnings_report() {
        let mut game = make_game();
        game.game_result.home.score = 1;
        game.game_result.away.score = 0;
        game.game_result.home.fan_factor = 2;
        game.game_result.away.fan_factor = 2;
        let mut step = StepWinnings;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::WINNINGS), "should add ReportWinnings");
    }

    #[test]
    fn winnings_report_added_for_both_teams() {
        let mut game = make_game();
        game.game_result.home.score = 2;
        game.game_result.away.score = 1;
        game.game_result.home.fan_factor = 4;
        game.game_result.away.fan_factor = 4;
        let mut step = StepWinnings;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::WINNINGS));
        assert_eq!(game.game_result.home.winnings, 60_000);
        assert_eq!(game.game_result.away.winnings, 50_000);
    }
}
