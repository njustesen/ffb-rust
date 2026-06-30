/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.end.StepFanFactor`.
///
/// Rolls post-match fan factor and applies the modifier to each team's result.
/// Java: `DiceRoller.rollFanFactorPostMatch(winner)` = 3D6 if winning, 2D6 otherwise.
/// Java: `DiceInterpreter.interpretFanFactorRoll(rolls, fan_factor, score_diff)`.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepFanFactor` (bb2016/end).
pub struct StepFanFactor;

impl StepFanFactor {
    pub fn new() -> Self { Self }

    /// Java: `DiceRoller.rollFanFactorPostMatch(pWinningTeam)`.
    /// Winning team rolls 3D6; loser rolls 2D6.
    fn roll_fan_factor_post_match(rng: &mut GameRng, winning: bool) -> (i32, i32, i32) {
        let a = rng.d6();
        let b = rng.d6();
        if winning {
            let c = rng.d6();
            (a, b, c)
        } else {
            (a, b, 0)
        }
    }

    fn sum(rolls: (i32, i32, i32), winning: bool) -> i32 {
        if winning { rolls.0 + rolls.1 + rolls.2 } else { rolls.0 + rolls.1 }
    }

    /// Java: `DiceInterpreter.interpretFanFactorRoll(rolls, fan_factor, score_diff)`.
    fn interpret_fan_factor_roll(total: i32, fan_factor: i32, score_diff: i32) -> i32 {
        if score_diff >= 0 && total > fan_factor {
            return 1;
        }
        if score_diff <= 0 && total < fan_factor {
            return -1;
        }
        0
    }

    fn execute_step(game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let score_diff_home = game.game_result.home.score - game.game_result.away.score;
        let home_fan_factor = game.team_home.fan_factor;
        let away_fan_factor = game.team_away.fan_factor;

        // Home
        let home_modifier = if !game.game_result.home.conceded {
            let rolls = Self::roll_fan_factor_post_match(rng, score_diff_home > 0);
            let total = Self::sum(rolls, score_diff_home > 0);
            Self::interpret_fan_factor_roll(total, home_fan_factor, score_diff_home)
        } else {
            -1
        };
        game.game_result.home.fan_factor_modifier = home_modifier;

        // Away (score_diff for away = -score_diff_home)
        let away_modifier = if !game.game_result.away.conceded {
            let rolls = Self::roll_fan_factor_post_match(rng, score_diff_home < 0);
            let total = Self::sum(rolls, score_diff_home < 0);
            Self::interpret_fan_factor_roll(total, away_fan_factor, -score_diff_home)
        } else {
            -1
        };
        game.game_result.away.fan_factor_modifier = away_modifier;

        StepOutcome::next()
    }
}

impl Default for StepFanFactor {
    fn default() -> Self { Self::new() }
}

impl Step for StepFanFactor {
    fn id(&self) -> StepId { StepId::FanFactor }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        Self::execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        Self::execute_step(game, rng)
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use ffb_model::enums::Rules;

    fn make_game(home_score: i32, away_score: i32, home_ff: i32, away_ff: i32) -> Game {
        let mut home = test_team("home", 0);
        let mut away = test_team("away", 0);
        home.fan_factor = home_ff;
        away.fan_factor = away_ff;
        let mut game = Game::new(home, away, Rules::Bb2016);
        game.game_result.home.score = home_score;
        game.game_result.away.score = away_score;
        game
    }

    #[test]
    fn id_is_fan_factor() {
        assert_eq!(StepFanFactor::new().id(), StepId::FanFactor);
    }

    #[test]
    fn interpret_roll_winning_above_ff_gives_plus_one() {
        // winning (diff >= 0), total > fan_factor → +1
        assert_eq!(StepFanFactor::interpret_fan_factor_roll(10, 5, 1), 1);
    }

    #[test]
    fn interpret_roll_losing_below_ff_gives_minus_one() {
        // losing (diff <= 0), total < fan_factor → -1
        assert_eq!(StepFanFactor::interpret_fan_factor_roll(3, 7, -1), -1);
    }

    #[test]
    fn interpret_roll_draw_no_change() {
        // draw (diff == 0), total == fan_factor → 0
        assert_eq!(StepFanFactor::interpret_fan_factor_roll(5, 5, 0), 0);
    }

    #[test]
    fn conceded_team_gets_minus_one() {
        let mut step = StepFanFactor::new();
        let mut game = make_game(3, 0, 5, 5);
        game.game_result.away.conceded = true;
        let mut rng = GameRng::new(42);
        step.start(&mut game, &mut rng);
        assert_eq!(game.game_result.away.fan_factor_modifier, -1);
    }

    #[test]
    fn modifiers_set_after_execute() {
        let mut step = StepFanFactor::new();
        let mut game = make_game(0, 0, 5, 5);
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        // Both should be set to -1, 0, or 1 — not the default uninitialised value
        assert!(game.game_result.home.fan_factor_modifier >= -1 && game.game_result.home.fan_factor_modifier <= 1);
        assert!(game.game_result.away.fan_factor_modifier >= -1 && game.game_result.away.fan_factor_modifier <= 1);
    }
}
