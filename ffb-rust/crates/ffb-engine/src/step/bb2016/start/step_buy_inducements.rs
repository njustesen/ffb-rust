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
/// TODO(BuyInducements-inducements): InducementTypeFactory, Inducement, InducementSet deferred.
/// TODO(BuyInducements-addStarPlayers): RosterPlayer creation + DB update deferred.
/// TODO(BuyInducements-addMercenaries): Loner skill injection deferred.
/// TODO(BuyInducements-generators): Inducement / RiotousRookies generators deferred.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

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

    fn execute_step(&mut self, _game: &mut Game) -> StepOutcome {
        // TODO(BuyInducements-options): check INDUCEMENTS option.
        // Auto-skip if under minimum.
        if self.inducement_gold_home < MINIMUM_PETTY_CASH_FOR_INDUCEMENTS {
            self.inducements_selected_home = true;
        }
        if self.inducement_gold_away < MINIMUM_PETTY_CASH_FOR_INDUCEMENTS {
            self.inducements_selected_away = true;
        }
        // TODO(BuyInducements-dialog): show dialog for teams still buying.
        // TODO(BuyInducements-generators): push Inducement + RiotousRookies sequences.
        if self.inducements_selected_home && self.inducements_selected_away {
            return StepOutcome::next();
        }
        StepOutcome::cont()
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
    fn both_rich_returns_continue() {
        let mut game = make_game();
        let mut step = StepBuyInducements::new();
        step.inducement_gold_home = 150_000;
        step.inducement_gold_away = 150_000;
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Dialog would be shown — until generator deferred, fall to Continue
        assert!(matches!(out.action, StepAction::Continue));
    }
}
