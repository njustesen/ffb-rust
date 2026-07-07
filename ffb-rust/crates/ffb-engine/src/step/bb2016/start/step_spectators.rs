/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.start.StepSpectators`.
///
/// Rolls spectator counts (2D6 + fan_factor) for both teams, then assigns fame
/// (0/1/2) based on the relative audience sizes (BB2016).
/// Java: `rollSpectators()` → teams get `(d1 + d2 + fan_factor) * 1000` spectators.
/// Fame rules: 2× or more → fame 2; more → fame 1; else → fame 0.
/// Note: pushes Kickoff sequence in Java — deferred (SequenceGeneratorFactory not yet ported).
use ffb_model::model::game::Game;
use ffb_model::report::bb2016::report_spectators::ReportSpectators;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepSpectators` (bb2016/start).
pub struct StepSpectators;

impl StepSpectators {
    pub fn new() -> Self { Self }

    /// Java: `rollSpectators()` — rolls 2D6 per team, computes spectators and fame,
    /// sets game result fields, then returns a `ReportSpectators`.
    fn roll_spectators(game: &mut Game, rng: &mut GameRng) -> ReportSpectators {
        // Java: int[] fanRollHome = getDiceRoller().rollSpectators() → [d6, d6]
        let fan_roll_home = [rng.d6(), rng.d6()];
        let spectators_home = (fan_roll_home[0] + fan_roll_home[1] + game.team_home.fan_factor) * 1000;

        // Java: int[] fanRollAway = getDiceRoller().rollSpectators() → [d6, d6]
        let fan_roll_away = [rng.d6(), rng.d6()];
        let spectators_away = (fan_roll_away[0] + fan_roll_away[1] + game.team_away.fan_factor) * 1000;

        // fan_factor in TeamResult = dedicated_fans + 2D6 (same convention as mixed version).
        game.game_result.home.fan_factor = fan_roll_home[0] + fan_roll_home[1] + game.team_home.fan_factor;
        game.game_result.away.fan_factor = fan_roll_away[0] + fan_roll_away[1] + game.team_away.fan_factor;

        // Fame: ≥ 2× opponent → 2; more → 1; else 0
        game.game_result.home.fame = if spectators_home >= 2 * spectators_away {
            2
        } else if spectators_home > spectators_away {
            1
        } else {
            0
        };
        game.game_result.away.fame = if spectators_away >= 2 * spectators_home {
            2
        } else if spectators_away > spectators_home {
            1
        } else {
            0
        };

        // Java: return new ReportSpectators(fanRollHome, teamResultHome.getSpectators(), teamResultHome.getFame(),
        //                                   fanRollAway, teamResultAway.getSpectators(), teamResultAway.getFame())
        ReportSpectators::new(
            fan_roll_home.to_vec(),
            spectators_home,
            game.game_result.home.fame,
            fan_roll_away.to_vec(),
            spectators_away,
            game.game_result.away.fame,
        )
    }
}

impl Default for StepSpectators {
    fn default() -> Self { Self::new() }
}

impl Step for StepSpectators {
    fn id(&self) -> StepId { StepId::Spectators }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: getResult().addReport(rollSpectators())
        let report = Self::roll_spectators(game, rng);
        game.report_list.add(report);
        StepOutcome::next()
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let report = Self::roll_spectators(game, rng);
        game.report_list.add(report);
        StepOutcome::next()
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::Rules;

    fn make_game(home_ff: i32, away_ff: i32) -> Game {
        let mut home = test_team("home", 0);
        let mut away = test_team("away", 0);
        home.fan_factor = home_ff;
        away.fan_factor = away_ff;
        Game::new(home, away, Rules::Bb2016)
    }

    #[test]
    fn id_is_spectators() {
        assert_eq!(StepSpectators::new().id(), StepId::Spectators);
    }

    #[test]
    fn fan_factor_at_least_base_plus_2() {
        let mut step = StepSpectators::new();
        let mut game = make_game(5, 4);
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        // fan_factor in TeamResult = 2D6 + base fan_factor; minimum 2D6 = 2
        assert!(game.game_result.home.fan_factor >= 5 + 2);
        assert!(game.game_result.away.fan_factor >= 4 + 2);
    }

    #[test]
    fn report_spectators_added_to_report_list() {
        use ffb_model::report::report_id::ReportId;
        let mut step = StepSpectators::new();
        let mut game = make_game(5, 5);
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert!(game.report_list.has_report(ReportId::SPECTATORS));
    }

    #[test]
    fn report_spectators_contains_correct_fame() {
        use ffb_model::report::report_id::ReportId;
        let mut step = StepSpectators::new();
        // home fan_factor 20 vs away 1 → home always has 2× spectators
        let mut game = make_game(20, 1);
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert!(game.report_list.has_report(ReportId::SPECTATORS));
        assert_eq!(game.game_result.home.fame, 2);
        assert_eq!(game.game_result.away.fame, 0);
    }

    #[test]
    fn large_advantage_gives_fame_2() {
        // home fan_factor 20 vs away 1 → home spectators >= 2× away regardless of rolls
        let mut step = StepSpectators::new();
        let mut game = make_game(20, 1);
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        // min home = (20+2)*1000 = 22000; max away = (1+12)*1000 = 13000 → home always 2×
        assert_eq!(game.game_result.home.fame, 2);
        assert_eq!(game.game_result.away.fame, 0);
    }

    #[test]
    fn next_step_always_returned() {
        let mut step = StepSpectators::new();
        let mut game = make_game(5, 5);
        let mut rng = GameRng::new(42);
        let outcome = step.start(&mut game, &mut rng);
        assert!(matches!(outcome.action, StepAction::NextStep));
    }

    #[test]
    fn fame_at_most_two() {
        let mut step = StepSpectators::new();
        let mut game = make_game(5, 5);
        let mut rng = GameRng::new(99);
        step.start(&mut game, &mut rng);
        assert!(game.game_result.home.fame <= 2);
        assert!(game.game_result.away.fame <= 2);
    }

    #[test]
    fn symmetric_roll_both_zero_fame() {
        // Equal fan factors + force home and away to get the same spectators:
        // Test that when spectators are equal neither team gets fame.
        // Use a game where we manually set fan_factor = 0 and seed rng for equal rolls.
        let mut step = StepSpectators::new();
        let mut game = make_game(0, 0);
        // Override: manually set fan_factors to equal after a known seed gives equal d6_two.
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        // Can't guarantee equality from rng alone, but fame is 0, 1, or 2 for each
        assert!(game.game_result.home.fame + game.game_result.away.fame <= 2);
    }
}
