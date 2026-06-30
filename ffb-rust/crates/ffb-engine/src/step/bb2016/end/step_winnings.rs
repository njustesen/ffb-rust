/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.end.StepWinnings`.
///
/// Step in end game sequence to roll winnings (BB2016).
/// - Rolls D6 for both teams; adds fame and +1 if tied/winning.
/// - Winner may re-roll via dialog (re-roll on a 1 or 2 in admin mode → auto re-roll).
/// - If team conceded illegally: transfer their winnings to the opposing team.
/// - Emits ReportWinningsRoll twice (initial + concede adjustment).
///
/// TODO(Winnings-dialog): DialogWinningsReRollParameter / UtilServerDialog deferred.
/// TODO(Winnings-reroll): AbstractStepWithReRoll / WINNINGS re-roll deferred.
/// TODO(Winnings-report): ReportWinningsRoll (currently just applies arithmetic).
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepWinnings` (bb2016/end).
pub struct StepWinnings;

impl StepWinnings {
    pub fn new() -> Self { Self }

    /// Java: `rollWinnings()` — rolls D6 for both teams and applies base winnings.
    fn roll_winnings(game: &mut Game, rng: &mut GameRng) -> (i32, i32) {
        let score_diff_home = game.game_result.home.score - game.game_result.away.score;
        // Roll home
        let roll_home = rng.d6();
        let winnings_home = roll_home + game.game_result.home.fame + if score_diff_home >= 0 { 1 } else { 0 };
        game.game_result.home.winnings = winnings_home * 10_000;
        // Roll away
        let roll_away = rng.d6();
        let winnings_away = roll_away + game.game_result.away.fame + if score_diff_home <= 0 { 1 } else { 0 };
        game.game_result.away.winnings = winnings_away * 10_000;
        (roll_home, roll_away)
    }

    /// Java: `concedeWinnings()` — transfer winnings on illegal concede.
    fn concede_winnings(game: &mut Game) {
        if game.game_result.home.conceded && !game.conceded_legally {
            game.game_result.away.winnings += game.game_result.home.winnings;
            game.game_result.home.winnings = 0;
        }
        if game.game_result.away.conceded && !game.conceded_legally {
            game.game_result.home.winnings += game.game_result.away.winnings;
            game.game_result.away.winnings = 0;
        }
    }

    fn execute_step(game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        Self::roll_winnings(game, rng);
        // TODO(Winnings-dialog): show re-roll dialog for winner.
        Self::concede_winnings(game);
        StepOutcome::next()
    }
}

impl Default for StepWinnings {
    fn default() -> Self { Self::new() }
}

impl Step for StepWinnings {
    fn id(&self) -> StepId { StepId::Winnings }

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

    fn make_game(home_score: i32, away_score: i32) -> Game {
        let mut game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016);
        game.game_result.home.score = home_score;
        game.game_result.away.score = away_score;
        game
    }

    #[test]
    fn id_is_winnings() {
        assert_eq!(StepWinnings::new().id(), StepId::Winnings);
    }

    #[test]
    fn start_returns_next_step() {
        let mut game = make_game(0, 0);
        let out = StepWinnings::new().start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
    }

    #[test]
    fn winnings_set_after_roll() {
        let mut game = make_game(0, 0);
        StepWinnings::new().start(&mut game, &mut GameRng::new(0));
        assert!(game.game_result.home.winnings > 0);
        assert!(game.game_result.away.winnings > 0);
    }

    #[test]
    fn home_illegal_concede_transfers_winnings() {
        let mut game = make_game(3, 1);
        game.game_result.home.conceded = true;
        game.conceded_legally = false;
        StepWinnings::new().start(&mut game, &mut GameRng::new(42));
        assert_eq!(game.game_result.home.winnings, 0);
        assert!(game.game_result.away.winnings > 0);
    }

    #[test]
    fn away_illegal_concede_transfers_winnings() {
        let mut game = make_game(1, 3);
        game.game_result.away.conceded = true;
        game.conceded_legally = false;
        StepWinnings::new().start(&mut game, &mut GameRng::new(42));
        assert_eq!(game.game_result.away.winnings, 0);
        assert!(game.game_result.home.winnings > 0);
    }

    #[test]
    fn winner_gets_extra_gold() {
        // home wins: score_diff_home > 0 → home_winnings += 1 extra
        let mut game = make_game(3, 0);
        game.game_result.home.fame = 0;
        game.game_result.away.fame = 0;
        StepWinnings::new().start(&mut game, &mut GameRng::new(1));
        // Minimum roll on D6 = 1; home gets 1+1 = 20000 minimum.
        assert!(game.game_result.home.winnings >= 20_000);
    }
}
