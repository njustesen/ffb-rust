/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.start.StepBuyInducements`.
///
/// Step in start game sequence to buy inducements (BB2016).
/// - Receives INDUCEMENT_GOLD_HOME / INDUCEMENT_GOLD_AWAY from preceding StepBuyCards.
/// - If INDUCEMENTS option disabled: skip to leaveStep.
/// - If USE_PREDEFINED_INDUCEMENTS: auto-apply team inducement sets.
/// - If gold < 50,000: mark that team done.
/// - Shows dialog for each team still buying.
/// - On CLIENT_BUY_INDUCEMENTS: apply inducement set, add star players / mercenaries.
/// - leaveStep: push Inducement + RiotousRookies sequences; record petty_cash_used.
///
/// Receives: INDUCEMENT_GOLD_HOME, INDUCEMENT_GOLD_AWAY.
///
/// headless: InducementTypeFactory, Inducement, InducementSet not yet ported.
/// headless: addStarPlayers — RosterPlayer creation + DB update not yet ported.
/// headless: addMercenaries — Loner skill injection not yet ported.
use ffb_model::enums::InducementPhase;
use ffb_model::model::game::Game;
use ffb_model::option::game_option_id::{INDUCEMENTS, USE_PREDEFINED_INDUCEMENTS};
use ffb_model::option::util_game_option::is_option_enabled;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::generator::common::inducement::{Inducement, InducementParams};
use crate::step::generator::common::riotous_rookies::RiotousRookies;
use crate::step::game::start::util_inducement_sequence::UtilInducementSequence;

const MINIMUM_PETTY_CASH_FOR_INDUCEMENTS: i32 = 50_000;

/// Java: `StepBuyInducements` (bb2016/start).
pub struct StepBuyInducements {
    /// Java: `fInducementGoldHome`
    inducement_gold_home: i32,
    /// Java: `fInducementGoldAway`
    inducement_gold_away: i32,
    /// Java: `fInducementsSelectedHome`
    inducements_selected_home: bool,
    /// Java: `fInducementsSelectedAway`
    inducements_selected_away: bool,
    /// Java: `fGoldUsedHome`
    gold_used_home: i32,
    /// Java: `fGoldUsedAway`
    gold_used_away: i32,
    /// Java: `fReportedHome`
    reported_home: bool,
    /// Java: `fReportedAway`
    reported_away: bool,
}

impl StepBuyInducements {
    pub fn new() -> Self {
        Self {
            inducement_gold_home: 0,
            inducement_gold_away: 0,
            inducements_selected_home: false,
            inducements_selected_away: false,
            gold_used_home: 0,
            gold_used_away: 0,
            reported_home: false,
            reported_away: false,
        }
    }

    fn execute_step(&mut self, game: &mut Game) -> StepOutcome {
        // Java: if (!INDUCEMENTS) → leaveStep (skip inducement buying entirely)
        if !is_option_enabled(game, INDUCEMENTS) {
            self.inducements_selected_home = true;
            self.inducements_selected_away = true;
            return self.leave_step(game);
        }

        // Java: if (USE_PREDEFINED_INDUCEMENTS) → apply predefined sets, skip dialog
        // headless: InducementTypeFactory not ported; treat as auto-skip
        if is_option_enabled(game, USE_PREDEFINED_INDUCEMENTS) {
            self.inducements_selected_home = true;
            self.inducements_selected_away = true;
            return self.leave_step(game);
        }
        // Auto-skip if under minimum.
        if self.inducement_gold_home < MINIMUM_PETTY_CASH_FOR_INDUCEMENTS {
            self.inducements_selected_home = true;
        }
        if self.inducement_gold_away < MINIMUM_PETTY_CASH_FOR_INDUCEMENTS {
            self.inducements_selected_away = true;
        }
        // client-only: show inducement buying dialog — headless auto-skips
        if self.inducements_selected_home && self.inducements_selected_away {
            return self.leave_step(game);
        }
        StepOutcome::cont()
    }

    fn leave_step(&self, game: &mut Game) -> StepOutcome {
        let home_tv = game.game_result.home.team_value;
        let away_tv = game.game_result.away.team_value;
        let (first_home, second_home) = if home_tv > away_tv { (true, false) } else { (false, true) };
        let seq1 = Inducement::build_sequence(&InducementParams {
            inducement_phase: InducementPhase::AfterInducementsPurchased,
            home_team: first_home,
            check_forgo: false,
        });
        let seq2 = Inducement::build_sequence(&InducementParams {
            inducement_phase: InducementPhase::AfterInducementsPurchased,
            home_team: second_home,
            check_forgo: false,
        });
        let seq_rr = RiotousRookies::build_sequence();
        // Java: game.getTeamHome/Away().getTeamData().setPettyCashUsed(UtilInducementSequence.calculateInducementGold(...))
        game.game_result.home.petty_cash_used = UtilInducementSequence::calculate_inducement_gold(Some(game), true);
        game.game_result.away.petty_cash_used = UtilInducementSequence::calculate_inducement_gold(Some(game), false);
        StepOutcome::next().push_seq(seq1).push_seq(seq2).push_seq(seq_rr)
    }
}

impl Default for StepBuyInducements {
    fn default() -> Self { Self::new() }
}

impl Step for StepBuyInducements {
    fn id(&self) -> StepId { StepId::BuyInducements }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::InducementGoldHome(v) => { self.inducement_gold_home = *v; true }
            StepParameter::InducementGoldAway(v) => { self.inducement_gold_away = *v; true }
            _ => false,
        }
    }
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
    fn id_is_buy_inducements() {
        assert_eq!(StepBuyInducements::new().id(), StepId::BuyInducements);
    }

    #[test]
    fn both_under_minimum_skips_to_next() {
        let mut game = make_game();
        let mut step = StepBuyInducements::new();
        // Both teams have 0 gold → both auto-selected
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
    }

    #[test]
    fn set_parameter_inducement_gold_home() {
        let mut step = StepBuyInducements::new();
        assert!(step.set_parameter(&StepParameter::InducementGoldHome(100_000)));
        assert_eq!(step.inducement_gold_home, 100_000);
    }

    #[test]
    fn set_parameter_inducement_gold_away() {
        let mut step = StepBuyInducements::new();
        assert!(step.set_parameter(&StepParameter::InducementGoldAway(75_000)));
        assert_eq!(step.inducement_gold_away, 75_000);
    }

    #[test]
    fn both_rich_returns_continue_when_inducements_enabled() {
        use ffb_model::option::game_option_id::INDUCEMENTS;
        let mut game = make_game();
        game.options.set(INDUCEMENTS, "true");
        let mut step = StepBuyInducements::new();
        step.inducement_gold_home = 150_000;
        step.inducement_gold_away = 150_000;
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Dialog would be shown — until generator deferred, fall to Continue
        assert!(matches!(out.action, StepAction::Continue));
    }

    #[test]
    fn both_under_minimum_pushes_three_sequences() {
        let mut game = make_game();
        let mut step = StepBuyInducements::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Two Inducement sequences + one RiotousRookies
        assert_eq!(out.pushes.len(), 3);
    }

    #[test]
    fn leave_step_sets_petty_cash_used() {
        let mut game = make_game();
        // Set TV diff so home gets petty cash
        game.game_result.home.team_value = 800_000;
        game.game_result.away.team_value = 1_000_000;
        game.game_result.away.petty_cash_transferred = 0;
        let mut step = StepBuyInducements::new();
        step.start(&mut game, &mut GameRng::new(0));
        // UtilInducementSequence: home gets 200k petty cash
        assert_eq!(game.game_result.home.petty_cash_used, 200_000);
        assert_eq!(game.game_result.away.petty_cash_used, 0);
    }

    #[test]
    fn inducements_disabled_skips_to_next_step() {
        let mut game = make_game();
        // INDUCEMENTS not set → disabled → skip to leaveStep immediately
        let mut step = StepBuyInducements::new();
        step.inducement_gold_home = 150_000;
        step.inducement_gold_away = 150_000;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
    }
}
