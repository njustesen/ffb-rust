/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.start.StepPettyCash`.
///
/// Calculates petty cash available to the underdog team from the TV difference.
/// Java: the `INDUCEMENTS_ALWAYS_USE_TREASURY` option check is omitted (not in Rust model).
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepPettyCash` (mixed/start, BB2020 + BB2025).
pub struct StepPettyCash;

impl StepPettyCash {
    pub fn new() -> Self { Self }

    fn execute_step(game: &mut Game) -> StepOutcome {
        let home_tv = game.team_home.team_value;
        let away_tv = game.team_away.team_value;

        // Java: update stored team value to the max seen so far
        game.game_result.home.team_value = i32::max(game.game_result.home.team_value, home_tv);
        game.game_result.away.team_value = i32::max(game.game_result.away.team_value, away_tv);

        let available_petty_cash = home_tv - away_tv;
        if available_petty_cash != 0 {
            // Negative means home is underdog; positive means away is underdog
            if available_petty_cash < 0 {
                game.game_result.home.petty_cash_from_tv_diff = available_petty_cash.unsigned_abs() as i32;
            } else {
                game.game_result.away.petty_cash_from_tv_diff = available_petty_cash;
            }
        }

        StepOutcome::next()
    }
}

impl Default for StepPettyCash {
    fn default() -> Self { Self::new() }
}

impl Step for StepPettyCash {
    fn id(&self) -> StepId { StepId::PettyCash }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        Self::execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        Self::execute_step(game)
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use ffb_model::enums::Rules;

    fn make_game(home_tv: i32, away_tv: i32) -> Game {
        let mut home = test_team("home", 0);
        let mut away = test_team("away", 0);
        home.team_value = home_tv;
        away.team_value = away_tv;
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn id_is_petty_cash() {
        assert_eq!(StepPettyCash::new().id(), StepId::PettyCash);
    }

    #[test]
    fn equal_tv_no_petty_cash() {
        let mut step = StepPettyCash::new();
        let mut game = make_game(1_000_000, 1_000_000);
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert_eq!(game.game_result.home.petty_cash_from_tv_diff, 0);
        assert_eq!(game.game_result.away.petty_cash_from_tv_diff, 0);
    }

    #[test]
    fn away_underdog_gets_petty_cash() {
        let mut step = StepPettyCash::new();
        let mut game = make_game(1_200_000, 1_000_000);
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert_eq!(game.game_result.away.petty_cash_from_tv_diff, 200_000);
        assert_eq!(game.game_result.home.petty_cash_from_tv_diff, 0);
    }

    #[test]
    fn home_underdog_gets_petty_cash() {
        let mut step = StepPettyCash::new();
        let mut game = make_game(800_000, 1_000_000);
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert_eq!(game.game_result.home.petty_cash_from_tv_diff, 200_000);
        assert_eq!(game.game_result.away.petty_cash_from_tv_diff, 0);
    }

    #[test]
    fn stores_team_value_in_result() {
        let mut step = StepPettyCash::new();
        let mut game = make_game(1_100_000, 900_000);
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert_eq!(game.game_result.home.team_value, 1_100_000);
        assert_eq!(game.game_result.away.team_value, 900_000);
    }
}
