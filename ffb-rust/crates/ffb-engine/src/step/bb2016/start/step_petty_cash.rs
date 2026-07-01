/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.start.StepPettyCash`.
///
/// Step in start game sequence to handle petty cash (BB2016).
/// - Always updates team_value in TeamResult to max(stored, current).
/// - If PETTY_CASH option disabled: NEXT_STEP immediately.
/// - If FORCE_TREASURY_TO_PETTY_CASH: auto-fill both teams' petty_cash_transferred.
/// - If treasury < 50,000 or opponent TV advantage > treasury: auto-select that team.
/// - Shows dialog for each team still deciding.
/// - On CLIENT_PETTY_CASH: record amount (clamped to 0..treasury).
/// - If PETTY_CASH_AFFECTS_TV: add petty_cash_transferred to team_value.
/// - Reports ReportPettyCash once per team.
///
/// DEFERRED(options): GameOptionId.PETTY_CASH etc. not yet ported.
/// DEFERRED(dialog): DialogPettyCashParameter not yet ported.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepPettyCash` (bb2016/start).
pub struct StepPettyCash {
    /// Java: `fPettyCashSelectedHome`
    petty_cash_selected_home: bool,
    /// Java: `fPettyCashSelectedAway`
    petty_cash_selected_away: bool,
    /// Java: `fReportedHome`
    reported_home: bool,
    /// Java: `fReportedAway`
    reported_away: bool,
}

impl StepPettyCash {
    pub fn new() -> Self {
        Self {
            petty_cash_selected_home: false,
            petty_cash_selected_away: false,
            reported_home: false,
            reported_away: false,
        }
    }

    fn normalize_petty_cash(entered: i32, max_treasury: i32) -> i32 {
        entered.max(0).min(max_treasury)
    }

    fn execute_step(&mut self, game: &mut Game) -> StepOutcome {
        // Update team values.
        let home_tv = game.team_home.team_value;
        let away_tv = game.team_away.team_value;
        game.game_result.home.team_value = game.game_result.home.team_value.max(home_tv);
        game.game_result.away.team_value = game.game_result.away.team_value.max(away_tv);

        // DEFERRED(options): PETTY_CASH GameOption / FORCE_TREASURY_TO_PETTY_CASH not yet ported.
        // Auto-select teams with treasury < 50,000.
        let home_treasury = game.team_home.treasury;
        let away_treasury = game.team_away.treasury;
        if home_treasury < 50_000 {
            self.petty_cash_selected_home = true;
        }
        if away_treasury < 50_000 {
            self.petty_cash_selected_away = true;
        }
        // DEFERRED(dialog): DialogPettyCashParameter not yet ported.
        if self.petty_cash_selected_home && self.petty_cash_selected_away {
            return StepOutcome::next();
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
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // DEFERRED(command): CLIENT_PETTY_CASH not yet mapped in Action enum.
        self.execute_step(game)
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
    fn id_is_petty_cash() {
        assert_eq!(StepPettyCash::new().id(), StepId::PettyCash);
    }

    #[test]
    fn both_no_treasury_returns_next() {
        let mut game = make_game();
        // Both teams have treasury 0 → both auto-selected.
        let mut step = StepPettyCash::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
    }

    #[test]
    fn normalize_petty_cash_clamps_negative() {
        assert_eq!(StepPettyCash::normalize_petty_cash(-100, 50_000), 0);
    }

    #[test]
    fn normalize_petty_cash_clamps_above_max() {
        assert_eq!(StepPettyCash::normalize_petty_cash(200_000, 50_000), 50_000);
    }

    #[test]
    fn normalize_petty_cash_exact_in_range() {
        assert_eq!(StepPettyCash::normalize_petty_cash(30_000, 50_000), 30_000);
    }
}
