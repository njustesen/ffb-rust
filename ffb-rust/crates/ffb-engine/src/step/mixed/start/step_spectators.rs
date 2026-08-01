/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.start.StepSpectators`.
///
/// Rolls fan factor for both teams at the start of the game (dedicated fans + a single D3).
/// Java: `rollFanFactor()` → `DiceRoller.rollFanFactor()` → `rollDice(3)` (one D3 per team).
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::report::mixed::report_fan_factor::ReportFanFactor;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepSpectators` (mixed/start, BB2020 + BB2025).
pub struct StepSpectators;

impl StepSpectators {
    pub fn new() -> Self { Self }

    fn roll_spectators(game: &mut Game, rng: &mut GameRng) {
        // Java mixed StepSpectators: `int fanRollHome = rollFanFactor()` = a single `rollDice(3)`.
        let fan_roll_home = rng.d3();
        game.game_result.home.fan_factor = game.team_home.dedicated_fans + fan_roll_home;

        let fan_roll_away = rng.d3();
        game.game_result.away.fan_factor = game.team_away.dedicated_fans + fan_roll_away;

        // Java: getResult().addReport(new ReportFanFactor(teamId, fanRoll, dedicatedFans)) — once per team
        game.report_list.add(ReportFanFactor::new(
            fan_roll_home,
            game.team_home.dedicated_fans,
            Some(game.team_home.id.clone()),
        ));
        game.report_list.add(ReportFanFactor::new(
            fan_roll_away,
            game.team_away.dedicated_fans,
            Some(game.team_away.id.clone()),
        ));
    }
}

impl Default for StepSpectators {
    fn default() -> Self { Self::new() }
}

impl Step for StepSpectators {
    fn id(&self) -> StepId { StepId::Spectators }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        Self::roll_spectators(game, rng);
        StepOutcome::next()
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        Self::roll_spectators(game, rng);
        StepOutcome::next()
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use ffb_model::enums::Rules;
    use ffb_model::report::report_id::ReportId;

    fn make_game(home_fans: i32, away_fans: i32) -> Game {
        let mut home = test_team("home", 0);
        let mut away = test_team("away", 0);
        home.dedicated_fans = home_fans;
        away.dedicated_fans = away_fans;
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn fan_factor_reports_added_for_both_teams() {
        let mut step = StepSpectators::new();
        let mut game = make_game(5, 4);
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert!(
            game.report_list.has_report(ReportId::FAN_FACTOR),
            "should add at least one ReportFanFactor"
        );
    }

    #[test]
    fn fan_factor_report_count_is_two() {
        let mut step = StepSpectators::new();
        let mut game = make_game(3, 2);
        let mut rng = GameRng::new(42);
        step.start(&mut game, &mut rng);
        let count = game.report_list.get_reports().iter()
            .filter(|r| r.get_id() == ReportId::FAN_FACTOR)
            .count();
        assert_eq!(count, 2, "should add exactly two ReportFanFactor (one per team)");
    }

    // Parity fix (Bug: bb2025 rolled 2d6): Java mixed StepSpectators rolls a SINGLE d3 per team.
    #[test]
    fn fan_factor_is_dedicated_fans_plus_one_d3_each() {
        let mut game = make_game(5, 4);
        let mut rng = GameRng::new(0);
        StepSpectators::roll_spectators(&mut game, &mut rng);
        // d3 range is 1..=3, so fan_factor - dedicated_fans ∈ [1,3] for each team.
        let home_roll = game.game_result.home.fan_factor - 5;
        let away_roll = game.game_result.away.fan_factor - 4;
        assert!((1..=3).contains(&home_roll), "home d3 roll {home_roll} out of 1..=3");
        assert!((1..=3).contains(&away_roll), "away d3 roll {away_roll} out of 1..=3");
    }

    #[test]
    fn consumes_exactly_one_d3_per_team_two_draws_total() {
        // Java rolls rollFanFactor() (one rollDice(3)) for home then away — exactly two dice, no more.
        // This is the crux of the parity fix: 2d6-per-team (4 draws) shifted the whole game-die stream.
        let mut game = make_game(0, 0);
        let mut rng = GameRng::new(7);
        assert_eq!(rng.call_count, 0);
        StepSpectators::roll_spectators(&mut game, &mut rng);
        assert_eq!(rng.call_count, 2, "must consume exactly 2 dice (one d3 per team), not 4 (old 2d6)");
    }

    #[test]
    fn fan_factor_bounds_for_zero_dedicated_fans() {
        let mut game = make_game(0, 0);
        let mut rng = GameRng::new(0);
        StepSpectators::roll_spectators(&mut game, &mut rng);
        assert!((1..=3).contains(&game.game_result.home.fan_factor));
        assert!((1..=3).contains(&game.game_result.away.fan_factor));
    }
}
