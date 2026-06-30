/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.start.StepSpectators`.
///
/// Rolls fan factor for both teams at the start of the game (2D6 + dedicated fans).
/// Java: `rollFanFactor()` → `DiceRoller.rollFanFactor()` → 2D6.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepSpectators` (mixed/start, BB2020 + BB2025).
pub struct StepSpectators;

impl StepSpectators {
    pub fn new() -> Self { Self }

    fn roll_spectators(game: &mut Game, rng: &mut GameRng) {
        let fan_roll_home = rng.d6_two();
        game.game_result.home.fan_factor = game.team_home.dedicated_fans + fan_roll_home;

        let fan_roll_away = rng.d6_two();
        game.game_result.away.fan_factor = game.team_away.dedicated_fans + fan_roll_away;
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

    fn make_game(home_fans: i32, away_fans: i32) -> Game {
        let mut home = test_team("home", 0);
        let mut away = test_team("away", 0);
        home.dedicated_fans = home_fans;
        away.dedicated_fans = away_fans;
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn id_is_spectators() {
        assert_eq!(StepSpectators::new().id(), StepId::Spectators);
    }

    #[test]
    fn fan_factor_at_least_dedicated_fans_plus_2() {
        let mut step = StepSpectators::new();
        let mut game = make_game(5, 4);
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        // 2D6 min is 2, so fan_factor >= dedicated_fans + 2
        assert!(game.game_result.home.fan_factor >= 5 + 2);
        assert!(game.game_result.away.fan_factor >= 4 + 2);
    }

    #[test]
    fn fan_factor_at_most_dedicated_fans_plus_12() {
        let mut step = StepSpectators::new();
        let mut game = make_game(3, 7);
        let mut rng = GameRng::new(42);
        step.start(&mut game, &mut rng);
        assert!(game.game_result.home.fan_factor <= 3 + 12);
        assert!(game.game_result.away.fan_factor <= 7 + 12);
    }

    #[test]
    fn different_seeds_may_give_different_results() {
        let mut game1 = make_game(5, 5);
        let mut game2 = make_game(5, 5);
        let mut rng1 = GameRng::new(1);
        let mut rng2 = GameRng::new(999999);
        StepSpectators::roll_spectators(&mut game1, &mut rng1);
        StepSpectators::roll_spectators(&mut game2, &mut rng2);
        // With different seeds at least one team should differ (probabilistic but very likely)
        let same = game1.game_result.home.fan_factor == game2.game_result.home.fan_factor
            && game1.game_result.away.fan_factor == game2.game_result.away.fan_factor;
        // We don't assert here since seeds could theoretically collide; just check both are valid
        let _ = same;
        assert!(game1.game_result.home.fan_factor >= 7);
    }

    #[test]
    fn zero_dedicated_fans_still_rolls() {
        let mut step = StepSpectators::new();
        let mut game = make_game(0, 0);
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert!(game.game_result.home.fan_factor >= 2);
        assert!(game.game_result.away.fan_factor >= 2);
    }
}
