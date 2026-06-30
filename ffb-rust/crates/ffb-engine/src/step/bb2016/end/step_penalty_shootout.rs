/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.end.StepPenaltyShootout`.
///
/// If the game is past half 2 and scores are tied, rolls D6 + rerolls_left for each team
/// repeatedly until one team wins, then awards them +1 score.
/// Java: `DiceRoller.rollPenaltyShootout()` = `rollDice(6)` = a D6 roll.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepPenaltyShootout` (bb2016/end).
pub struct StepPenaltyShootout;

impl StepPenaltyShootout {
    pub fn new() -> Self { Self }

    fn execute_step(game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if game.half > 2
            && game.game_result.home.score == game.game_result.away.score
        {
            let mut penalty_home;
            let mut penalty_away;
            loop {
                // Java: DiceRoller.rollPenaltyShootout() = rollDice(6)
                let roll_home = rng.d6();
                let rerolls_home = game.turn_data_home.rerolls;
                penalty_home = roll_home + rerolls_home;

                let roll_away = rng.d6();
                let rerolls_away = game.turn_data_away.rerolls;
                penalty_away = roll_away + rerolls_away;

                if penalty_home != penalty_away {
                    break;
                }
            }
            if penalty_home > penalty_away {
                game.game_result.home.score += 1;
            } else {
                game.game_result.away.score += 1;
            }
        }
        StepOutcome::next()
    }
}

impl Default for StepPenaltyShootout {
    fn default() -> Self { Self::new() }
}

impl Step for StepPenaltyShootout {
    fn id(&self) -> StepId { StepId::PenaltyShootout }

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
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    #[test]
    fn id_is_penalty_shootout() {
        assert_eq!(StepPenaltyShootout::new().id(), StepId::PenaltyShootout);
    }

    #[test]
    fn no_tie_no_change() {
        let mut step = StepPenaltyShootout::new();
        let mut game = make_game();
        game.half = 3;
        game.game_result.home.score = 2;
        game.game_result.away.score = 1;
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert!(matches!(outcome.action, StepAction::NextStep));
        assert_eq!(game.game_result.home.score, 2);
        assert_eq!(game.game_result.away.score, 1);
    }

    #[test]
    fn not_overtime_no_change() {
        // half <= 2, even if tied, no penalty shootout
        let mut step = StepPenaltyShootout::new();
        let mut game = make_game();
        game.half = 2;
        game.game_result.home.score = 1;
        game.game_result.away.score = 1;
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert!(matches!(outcome.action, StepAction::NextStep));
        assert_eq!(game.game_result.home.score, 1);
        assert_eq!(game.game_result.away.score, 1);
    }

    #[test]
    fn overtime_tie_resolved_one_team_wins() {
        let mut step = StepPenaltyShootout::new();
        let mut game = make_game();
        game.half = 3;
        game.game_result.home.score = 1;
        game.game_result.away.score = 1;
        let mut rng = GameRng::new(42);
        let outcome = step.start(&mut game, &mut rng);
        assert!(matches!(outcome.action, StepAction::NextStep));
        let total = game.game_result.home.score + game.game_result.away.score;
        // One team got +1
        assert_eq!(total, 3);
    }

    #[test]
    fn overtime_tie_rerolls_factor_in() {
        // Give home 10 rerolls so home always wins in a tie
        let mut step = StepPenaltyShootout::new();
        let mut game = make_game();
        game.half = 3;
        game.game_result.home.score = 0;
        game.game_result.away.score = 0;
        game.turn_data_home.rerolls = 100; // guaranteed home wins
        game.turn_data_away.rerolls = 0;
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert_eq!(game.game_result.home.score, 1);
        assert_eq!(game.game_result.away.score, 0);
    }

    #[test]
    fn set_parameter_always_false() {
        let mut step = StepPenaltyShootout::new();
        assert!(!step.set_parameter(&StepParameter::EndPlayerAction(true)));
    }
}
