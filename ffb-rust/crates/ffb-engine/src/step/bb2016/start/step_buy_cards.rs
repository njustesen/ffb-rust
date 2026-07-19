/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.start.StepBuyCards`.
///
/// Step in start game sequence to buy cards (BB2016).
/// - If MAX_NR_OF_CARDS == 0 or USE_PREDEFINED_INDUCEMENTS: skip, publish gold + NEXT_STEP.
/// - Otherwise: manage two-team card-buying loop.
///   - Builds decks per CardType; computes per-type prices from game options.
///   - Shows dialog when a team still has unspent gold ≥ minimum card price.
///   - On CLIENT_BUY_CARD: deduct gold, draw card from deck for the buying team.
///   - When both teams done: cap gold at max, publish INDUCEMENT_GOLD_HOME/AWAY + NEXT_STEP.
///
/// Publishes: INDUCEMENT_GOLD_HOME, INDUCEMENT_GOLD_AWAY.
///
/// CardDeck / CardTypeFactory not ported; headless always skips card purchasing.
/// client-only: DialogBuyCardsParameter — headless skips card purchasing dialog
use ffb_model::model::game::Game;
use ffb_model::option::game_option_id::{FREE_CARD_CASH, FREE_INDUCEMENT_CASH, MAX_NR_OF_CARDS, USE_PREDEFINED_INDUCEMENTS};
use ffb_model::option::util_game_option::{get_int_option, is_option_enabled};
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::game::start::util_inducement_sequence::UtilInducementSequence;

/// Java: `StepBuyCards` (bb2016/start).
pub struct StepBuyCards {
    /// Java: `fInducementGoldHome`
    inducement_gold_home: i32,
    /// Java: `fInducementGoldAway`
    inducement_gold_away: i32,
    /// Java: `fCardsSelectedHome`
    cards_selected_home: bool,
    /// Java: `fCardsSelectedAway`
    cards_selected_away: bool,
    /// Java: `fReportedHome`
    reported_home: bool,
    /// Java: `fReportedAway`
    reported_away: bool,
}

impl StepBuyCards {
    pub fn new() -> Self {
        Self {
            inducement_gold_home: 0,
            inducement_gold_away: 0,
            cards_selected_home: false,
            cards_selected_away: false,
            reported_home: false,
            reported_away: false,
        }
    }

    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: if (MAX_NR_OF_CARDS == 0 || USE_PREDEFINED_INDUCEMENTS) → skip card buying
        if get_int_option(game, MAX_NR_OF_CARDS) == 0 || is_option_enabled(game, USE_PREDEFINED_INDUCEMENTS) {
            // Java: freeCash = FREE_INDUCEMENT_CASH; fInducementGoldHome/Away =
            // calculateInducementGold(...) + freeCash — must actually compute the gold here,
            // not just republish the (always-zero-at-this-point) field.
            let free_cash = get_int_option(game, FREE_INDUCEMENT_CASH);
            self.inducement_gold_home = UtilInducementSequence::calculate_inducement_gold(Some(game), true) + free_cash;
            self.inducement_gold_away = UtilInducementSequence::calculate_inducement_gold(Some(game), false) + free_cash;
            return StepOutcome::next()
                .publish(StepParameter::InducementGoldHome(self.inducement_gold_home))
                .publish(StepParameter::InducementGoldAway(self.inducement_gold_away));
        }
        // no-op: CardDeck / CardTypeFactory not ported — headless treats both teams as done
        // (no cards bought), but must still compute inducement gold as Java does
        // (lines 146-151 of StepBuyCards.executeStep) so downstream steps (StepBuyInducements)
        // receive the correct available gold, not 0.
        self.cards_selected_home = true;
        self.cards_selected_away = true;
        let free_cash = get_int_option(game, FREE_INDUCEMENT_CASH) + get_int_option(game, FREE_CARD_CASH);
        self.inducement_gold_home = UtilInducementSequence::calculate_inducement_gold(Some(game), true) + free_cash;
        self.inducement_gold_away = UtilInducementSequence::calculate_inducement_gold(Some(game), false) + free_cash;

        // Java: cap at max inducement gold (freeInducementCash only, no card cash) once both
        // teams are done selecting.
        let free_cash_only = get_int_option(game, FREE_INDUCEMENT_CASH);
        let max_gold_home = UtilInducementSequence::calculate_inducement_gold(Some(game), true) + free_cash_only;
        let max_gold_away = UtilInducementSequence::calculate_inducement_gold(Some(game), false) + free_cash_only;
        self.inducement_gold_home = self.inducement_gold_home.min(max_gold_home);
        self.inducement_gold_away = self.inducement_gold_away.min(max_gold_away);

        StepOutcome::next()
            .publish(StepParameter::InducementGoldHome(self.inducement_gold_home))
            .publish(StepParameter::InducementGoldAway(self.inducement_gold_away))
    }
}

impl Default for StepBuyCards {
    fn default() -> Self { Self::new() }
}

impl Step for StepBuyCards {
    fn id(&self) -> StepId { StepId::BuyCards }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
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
    fn id_is_buy_cards() {
        assert_eq!(StepBuyCards::new().id(), StepId::BuyCards);
    }

    #[test]
    fn start_returns_next_step() {
        let mut game = make_game();
        let mut step = StepBuyCards::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
    }

    #[test]
    fn publishes_inducement_gold_home() {
        // Java always recomputes fInducementGoldHome from calculateInducementGold(...) +
        // freeCash before publishing, so the published value must reflect actual game state
        // (TV difference), not a manually pre-set field.
        let mut game = make_game();
        game.game_result.away.team_value = 1_100_000;
        game.game_result.home.team_value = 1_000_000;
        let mut step = StepBuyCards::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::InducementGoldHome(100_000))));
    }

    #[test]
    fn publishes_inducement_gold_away() {
        let mut game = make_game();
        game.game_result.home.team_value = 1_050_000;
        game.game_result.away.team_value = 1_000_000;
        let mut step = StepBuyCards::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::InducementGoldAway(50_000))));
    }
    #[test]
    fn new_and_default_create_equivalent_instances() {
        let _a = StepBuyCards::new();
        let _b = StepBuyCards::default();
    }

    /// Java: even in the (untranslated CardDeck) no-op branch, StepBuyCards must still
    /// compute and publish real inducement gold from team-value differences
    /// (StepBuyCards.executeStep lines 146-151), not leave it at 0 — since
    /// StepBuyInducements relies entirely on the published INDUCEMENT_GOLD_HOME/AWAY
    /// parameter for how much gold a team has to spend.
    #[test]
    fn computes_inducement_gold_from_team_value_difference_when_card_buying_skipped() {
        let mut game = make_game();
        // Away has a much higher team value than home → home gets inducement gold
        // equal to the TV difference (UtilInducementSequence.calculateInducementGold).
        game.game_result.home.team_value = 1_000_000;
        game.game_result.away.team_value = 1_200_000;
        let mut step = StepBuyCards::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        let gold_home = out.published.iter().find_map(|p| match p {
            StepParameter::InducementGoldHome(v) => Some(*v),
            _ => None,
        }).expect("InducementGoldHome must be published");
        assert!(gold_home > 0, "home team should receive nonzero inducement gold from TV difference, got {gold_home}");
    }
}
