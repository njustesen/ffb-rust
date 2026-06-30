use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::StepId;

/// Calculates and sets each team's winnings at the end of the game.
/// Formula: (score + 1 unless stalled) + attendance share, × 10,000 gp.
/// On illegal concession the winning team receives all attendance.
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.end.StepWinnings`.
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
        // Java: attendance = gameResult.getTeamResultAway().getFanFactor() + getTeamResultHome().getFanFactor()
        // This is TeamResult.fan_factor, not Team.fan_factor — they may differ after FAME calculation.
        let attendance = (game.game_result.home.fan_factor + game.game_result.away.fan_factor) as f64;
        let mut home_winnings = game.game_result.home.score as f64;
        let mut away_winnings = game.game_result.away.score as f64;

        if !game.game_result.home.stalled { home_winnings += 1.0; }
        if !game.game_result.away.stalled { away_winnings += 1.0; }

        if game.game_result.home.conceded && !game.conceded_legally {
            away_winnings += attendance;
            home_winnings = 0.0;
        } else if game.game_result.away.conceded && !game.conceded_legally {
            home_winnings += attendance;
            away_winnings = 0.0;
        } else {
            home_winnings += attendance / 2.0;
            away_winnings += attendance / 2.0;
        }

        game.game_result.home.winnings = (home_winnings * 10_000.0) as i32;
        game.game_result.away.winnings = (away_winnings * 10_000.0) as i32;

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
        Game::new(home, away, Rules::Bb2025)
    }

    /// Symmetric normal game: each team gets score+1 + half of attendance.
    /// Home: 2 TDs, away: 1 TD, each fan_factor = 5  → attendance = 10.
    /// home_winnings = (2+1+5) * 10_000 = 80_000
    /// away_winnings = (1+1+5) * 10_000 = 70_000
    #[test]
    fn normal_game_splits_attendance_evenly() {
        let mut game = make_game();
        let mut step = StepWinnings;
        game.game_result.home.score = 2;
        game.game_result.away.score = 1;
        game.game_result.home.fan_factor = 5;
        game.game_result.away.fan_factor = 5;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.game_result.home.winnings, 80_000);
        assert_eq!(game.game_result.away.winnings, 70_000);
    }

    /// Stalled team loses the +1 bonus.
    #[test]
    fn stalled_team_loses_score_bonus() {
        let mut game = make_game();
        let mut step = StepWinnings;
        game.game_result.home.score = 1;
        game.game_result.away.score = 1;
        game.game_result.home.stalled = true;
        // attendance = 0, home: 1 (no +1), away: 1+1 = 2
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.game_result.home.winnings, 10_000);
        assert_eq!(game.game_result.away.winnings, 20_000);
    }

    /// Illegal home concession: away gets all attendance, home gets 0.
    /// home score=0 (conceded), away score=2. fan_factors: home=3, away=4 → attendance=7.
    /// away = (2+1+7)*10_000 = 100_000, home = 0.
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
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.game_result.home.winnings, 0);
        assert_eq!(game.game_result.away.winnings, 100_000); // (2+1+7)*10_000
    }

    /// Illegal away concession: home gets all attendance, away gets 0.
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
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.game_result.away.winnings, 0);
        assert_eq!(game.game_result.home.winnings, 100_000); // (2+1+7)*10_000
    }

    /// Legal concession still splits attendance normally.
    #[test]
    fn legal_concession_splits_attendance_normally() {
        let mut game = make_game();
        let mut step = StepWinnings;
        game.game_result.home.conceded = true;
        game.conceded_legally = true;
        game.game_result.home.score = 1;
        game.game_result.away.score = 2;
        game.game_result.home.fan_factor = 2;
        game.game_result.away.fan_factor = 2;
        // attendance=4, home=1+1+2=4 → 40_000, away=2+1+2=5 → 50_000
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.game_result.home.winnings, 40_000);
        assert_eq!(game.game_result.away.winnings, 50_000);
    }

    /// Odd attendance (e.g. 5) is split as floats — Java uses double division so each team
    /// gets 2.5 → truncated to 25_000 per team.
    #[test]
    fn odd_attendance_truncates_correctly() {
        let mut game = make_game();
        let mut step = StepWinnings;
        game.game_result.home.score = 0;
        game.game_result.away.score = 0;
        game.game_result.home.fan_factor = 3;
        game.game_result.away.fan_factor = 2; // total = 5, each gets 2.5 → 25_000
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        // 0+1+2.5 = 3.5, cast to i32 = 3 → 35_000
        assert_eq!(game.game_result.home.winnings, 35_000);
        assert_eq!(game.game_result.away.winnings, 35_000);
    }
}
