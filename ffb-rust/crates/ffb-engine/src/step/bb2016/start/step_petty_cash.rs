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
/// client-only: DialogPettyCashParameter — headless skips petty cash dialog
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::option::game_option_id::{PETTY_CASH, FORCE_TREASURY_TO_PETTY_CASH, PETTY_CASH_AFFECTS_TV};
use ffb_model::option::util_game_option::is_option_enabled;
use ffb_model::report::report_petty_cash::ReportPettyCash;
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
        let mut pending_events: Vec<GameEvent> = vec![];
        // Update team values.
        let home_tv = game.team_home.team_value;
        let away_tv = game.team_away.team_value;
        game.game_result.home.team_value = game.game_result.home.team_value.max(home_tv);
        game.game_result.away.team_value = game.game_result.away.team_value.max(away_tv);

        // Java: if (!UtilGameOption.isOptionEnabled(game, GameOptionId.PETTY_CASH)) return NEXT_STEP
        if !is_option_enabled(game, PETTY_CASH) {
            return StepOutcome::next();
        }

        let home_treasury = game.team_home.treasury;
        let away_treasury = game.team_away.treasury;

        // Java: FORCE_TREASURY_TO_PETTY_CASH — auto-fill both teams with treasury + TV deficit
        if is_option_enabled(game, FORCE_TREASURY_TO_PETTY_CASH) {
            let home_tv = game.team_home.team_value;
            let away_tv = game.team_away.team_value;
            game.game_result.home.petty_cash_transferred =
                home_treasury + (away_tv - home_tv).max(0);
            game.game_result.away.petty_cash_transferred =
                away_treasury + (home_tv - away_tv).max(0);
            self.petty_cash_selected_home = true;
            self.petty_cash_selected_away = true;
        }

        // Java: auto-select teams with treasury < 50,000 or TV disadvantage > treasury
        let home_tv = game.team_home.team_value;
        let away_tv = game.team_away.team_value;
        if !self.petty_cash_selected_home
            && (home_treasury < 50_000
                || (self.petty_cash_selected_away && (away_tv - home_tv) > home_treasury))
        {
            self.petty_cash_selected_home = true;
        }
        if !self.petty_cash_selected_away
            && (away_treasury < 50_000
                || (self.petty_cash_selected_home && (home_tv - away_tv) > away_treasury))
        {
            self.petty_cash_selected_away = true;
        }

        // Java: report + PETTY_CASH_AFFECTS_TV for each newly selected team
        if self.petty_cash_selected_home && !self.reported_home {
            if is_option_enabled(game, PETTY_CASH_AFFECTS_TV) {
                let transfer = game.game_result.home.petty_cash_transferred;
                game.game_result.home.team_value += transfer;
            }
            // Java: getResult().addReport(new ReportPettyCash(game.getTeamHome().getId(), ...))
            game.report_list.add(ReportPettyCash::new(
                game.team_home.id.clone(),
                game.game_result.home.petty_cash_transferred,
            ));
            pending_events.push(GameEvent::PettyCash {
                team_id: game.team_home.id.clone(),
                amount: game.game_result.home.petty_cash_transferred,
            });
            self.reported_home = true;
        }
        if self.petty_cash_selected_away && !self.reported_away {
            if is_option_enabled(game, PETTY_CASH_AFFECTS_TV) {
                let transfer = game.game_result.away.petty_cash_transferred;
                game.game_result.away.team_value += transfer;
            }
            // Java: getResult().addReport(new ReportPettyCash(game.getTeamAway().getId(), ...))
            game.report_list.add(ReportPettyCash::new(
                game.team_away.id.clone(),
                game.game_result.away.petty_cash_transferred,
            ));
            pending_events.push(GameEvent::PettyCash {
                team_id: game.team_away.id.clone(),
                amount: game.game_result.away.petty_cash_transferred,
            });
            self.reported_away = true;
        }

        let mut out = if self.petty_cash_selected_home && self.petty_cash_selected_away {
            StepOutcome::next()
        } else {
            // client-only: DialogPettyCashParameter — headless auto-skips
            StepOutcome::next()
        };
        for ev in pending_events { out = out.with_event(ev); }
        out
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

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        if let Action::PettyCash { home, amount } = action {
            if *home {
                let capped = Self::normalize_petty_cash(*amount, game.team_home.treasury);
                game.game_result.home.petty_cash_transferred = capped;
                self.petty_cash_selected_home = true;
            } else {
                let capped = Self::normalize_petty_cash(*amount, game.team_away.treasury);
                game.game_result.away.petty_cash_transferred = capped;
                self.petty_cash_selected_away = true;
            }
        }
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
    use ffb_model::option::game_option_id::{PETTY_CASH, FORCE_TREASURY_TO_PETTY_CASH, PETTY_CASH_AFFECTS_TV};

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
        game.options.set(PETTY_CASH, "true");
        // Both teams have treasury 0 → both auto-selected.
        let mut step = StepPettyCash::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
    }

    #[test]
    fn petty_cash_option_disabled_skips_immediately() {
        let mut game = make_game();
        // PETTY_CASH not set → disabled → NEXT_STEP
        game.team_home.treasury = 200_000;
        game.team_away.treasury = 200_000;
        let mut step = StepPettyCash::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
        // No petty cash set
        assert_eq!(game.game_result.home.petty_cash_transferred, 0);
    }

    #[test]
    fn force_treasury_to_petty_cash_auto_fills_both_teams() {
        let mut game = make_game();
        game.options.set(PETTY_CASH, "true");
        game.options.set(FORCE_TREASURY_TO_PETTY_CASH, "true");
        game.team_home.treasury = 100_000;
        game.team_away.treasury = 80_000;
        game.team_home.team_value = 1_000_000;
        game.team_away.team_value = 900_000;
        let mut step = StepPettyCash::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Home: treasury + max(0, away_tv - home_tv) = 100,000 + max(0, -100,000) = 100,000
        assert_eq!(game.game_result.home.petty_cash_transferred, 100_000);
        // Away: treasury + max(0, home_tv - away_tv) = 80,000 + 100,000 = 180,000
        assert_eq!(game.game_result.away.petty_cash_transferred, 180_000);
        assert!(matches!(out.action, StepAction::NextStep));
    }

    #[test]
    fn petty_cash_affects_tv_adds_transfer_to_team_value() {
        let mut game = make_game();
        game.options.set(PETTY_CASH, "true");
        game.options.set(PETTY_CASH_AFFECTS_TV, "true");
        // Both teams have treasury 0 → auto-selected with petty_cash_transferred=0
        game.team_home.team_value = 1_000_000;
        game.team_away.team_value = 1_000_000;
        let mut step = StepPettyCash::new();
        step.start(&mut game, &mut GameRng::new(0));
        // team_value stored in result: max(result_tv, team_tv) + petty_cash_transferred(0)
        assert_eq!(game.game_result.home.team_value, 1_000_000);
    }

    #[test]
    fn petty_cash_affects_tv_adds_nonzero_transfer() {
        let mut game = make_game();
        game.options.set(PETTY_CASH, "true");
        game.options.set(FORCE_TREASURY_TO_PETTY_CASH, "true");
        game.options.set(PETTY_CASH_AFFECTS_TV, "true");
        game.team_home.treasury = 50_000;
        game.team_away.treasury = 50_000;
        game.team_home.team_value = 1_000_000;
        game.team_away.team_value = 1_000_000;
        let mut step = StepPettyCash::new();
        step.start(&mut game, &mut GameRng::new(0));
        // team_value = 1_000_000 (stored) + 50_000 (transfer)
        assert_eq!(game.game_result.home.team_value, 1_050_000);
        assert_eq!(game.game_result.away.team_value, 1_050_000);
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

    #[test]
    fn petty_cash_command_home_sets_transferred() {
        let mut game = make_game();
        game.team_home.treasury = 100_000;
        let mut step = StepPettyCash::new();
        step.handle_command(
            &Action::PettyCash { home: true, amount: 40_000 },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(game.game_result.home.petty_cash_transferred, 40_000);
    }

    #[test]
    fn petty_cash_command_away_sets_transferred() {
        let mut game = make_game();
        game.team_away.treasury = 80_000;
        let mut step = StepPettyCash::new();
        step.handle_command(
            &Action::PettyCash { home: false, amount: 60_000 },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(game.game_result.away.petty_cash_transferred, 60_000);
    }

    #[test]
    fn report_petty_cash_added_for_both_teams_when_auto_selected() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        game.options.set(PETTY_CASH, "true");
        // Both teams have 0 treasury → both auto-selected
        let mut step = StepPettyCash::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.report_list.size(), 2, "expect one report per team");
        assert!(game.report_list.has_report(ReportId::PETTY_CASH));
    }

    #[test]
    fn report_petty_cash_not_added_when_option_disabled() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        // PETTY_CASH option disabled → NEXT_STEP immediately, no report
        let mut step = StepPettyCash::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!game.report_list.has_report(ReportId::PETTY_CASH));
    }

    #[test]
    fn petty_cash_command_clamps_to_treasury() {
        let mut game = make_game();
        game.team_home.treasury = 30_000;
        let mut step = StepPettyCash::new();
        step.handle_command(
            &Action::PettyCash { home: true, amount: 999_999 },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(game.game_result.home.petty_cash_transferred, 30_000);
    }
}
