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
/// The dialog loop is ported (AgentPrompt::BuyCards); CardDeck / CardFactory are not, because both
/// parity agents always decline -- a card purchase panics rather than silently diverge.
use ffb_model::model::game::Game;
use ffb_model::option::game_option_id::{CARDS_DIRTY_TRICK_COST, CARDS_MAGIC_ITEM_COST, FREE_CARD_CASH, FREE_INDUCEMENT_CASH, MAX_NR_OF_CARDS, USE_PREDEFINED_INDUCEMENTS};
use ffb_model::prompts::AgentPrompt;
use ffb_model::option::util_game_option::{get_int_option, is_option_enabled};
use ffb_model::report::bb2016::report_cards_bought::ReportCardsBought;
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
        // Java `buildDecks()` + the price scan: every bb2016 CardType (MAGIC_ITEM, DIRTY_TRICK)
        // has a full deck before any purchase (CardDeck.build adds every card of the factory and
        // removes only cards already in an inducement set -- none at game start), so the minimum
        // price is the cheaper of the two cost options. Building decks draws no dice.
        let minimum_card_price = get_int_option(game, CARDS_MAGIC_ITEM_COST)
            .min(get_int_option(game, CARDS_DIRTY_TRICK_COST));

        // Java: `if (fBuyCardHome != null) {...} else if (fBuyCardAway != null) {...}` -- a card
        // purchase draws from the deck. Both agents always decline (pregame "purchases declined"
        // contract, ParityRunner BUY_CARDS arm), so no card is ever bought; a non-null buy would
        // need the CardDeck/CardFactory port and is rejected in handle_command.
        if !self.cards_selected_home && !self.cards_selected_away {
            let free_cash = get_int_option(game, FREE_INDUCEMENT_CASH) + get_int_option(game, FREE_CARD_CASH);
            self.inducement_gold_home = UtilInducementSequence::calculate_inducement_gold(Some(game), true) + free_cash;
            self.inducement_gold_away = UtilInducementSequence::calculate_inducement_gold(Some(game), false) + free_cash;
        }
        if self.inducement_gold_home < minimum_card_price {
            self.cards_selected_home = true;
        }
        if self.inducement_gold_away < minimum_card_price {
            self.cards_selected_away = true;
        }
        // Java: `ReportCardsBought(team, cards.length, totalCost)` once per team when it is done.
        // No card is ever drawn (see above), so both report 0 cards for 0 gold.
        if self.cards_selected_home && !self.reported_home {
            self.reported_home = true;
            game.report_list.add(ReportCardsBought::new(game.team_home.id.clone(), 0, 0));
        }
        if self.cards_selected_away && !self.reported_away {
            self.reported_away = true;
            game.report_list.add(ReportCardsBought::new(game.team_away.id.clone(), 0, 0));
        }
        if !self.cards_selected_home && !self.cards_selected_away {
            // Java: the HIGHER team value is asked first (strict >, a tie asks away).
            let home_tv = game.game_result.home.team_value;
            let away_tv = game.game_result.away.team_value;
            return if home_tv > away_tv { self.dialog(game, true) } else { self.dialog(game, false) };
        } else if !self.cards_selected_home {
            return self.dialog(game, true);
        } else if !self.cards_selected_away {
            return self.dialog(game, false);
        }

        // Java: cap at max inducement gold (freeInducementCash only, no card cash) once both
        // teams are done selecting.
        let free_cash_only = get_int_option(game, FREE_INDUCEMENT_CASH);
        let max_gold_home = UtilInducementSequence::calculate_inducement_gold(Some(game), true) + free_cash_only;
        let max_gold_away = UtilInducementSequence::calculate_inducement_gold(Some(game), false) + free_cash_only;
        StepOutcome::next()
            .publish(StepParameter::InducementGoldHome(self.inducement_gold_home.min(max_gold_home)))
            .publish(StepParameter::InducementGoldAway(self.inducement_gold_away.min(max_gold_away)))
    }

    /// Java: `UtilServerDialog.showDialog(createDialogParameter(teamId, availableGold))`.
    fn dialog(&self, game: &Game, home: bool) -> StepOutcome {
        let (team_id, gold) = if home {
            (game.team_home.id.clone(), self.inducement_gold_home)
        } else {
            (game.team_away.id.clone(), self.inducement_gold_away)
        };
        StepOutcome::cont().with_prompt(AgentPrompt::BuyCards { team_id, available_gold: gold })
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

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: CLIENT_BUY_CARD -- the team is keyed on the sending session; a null card type
        // marks it done. Anything else is not this step's command (Java: UNHANDLED_COMMAND).
        match action {
            Action::BuyCard { home, card_type: None } => {
                if *home { self.cards_selected_home = true; } else { self.cards_selected_away = true; }
                self.execute_step(game, rng)
            }
            Action::BuyCard { card_type: Some(t), .. } =>
                panic!("StepBuyCards: buying card type {t} needs the CardDeck port (agents always decline)"),
            _ => StepOutcome::cont(),
        }
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::prompts::AgentPrompt;
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    #[test]
    fn start_returns_next_step() {
        let mut game = make_game();
        let mut step = StepBuyCards::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
    }

    fn gold(out: &StepOutcome) -> (Option<i32>, Option<i32>) {
        let h = out.published.iter().find_map(|p| match p { StepParameter::InducementGoldHome(v) => Some(*v), _ => None });
        let a = out.published.iter().find_map(|p| match p { StepParameter::InducementGoldAway(v) => Some(*v), _ => None });
        (h, a)
    }

    /// A gap below the cheapest card (50k): Java marks both teams done in the first pass, reports
    /// 0 cards for each, and publishes the gold with no dialog.
    #[test]
    fn gap_below_cheapest_card_skips_the_dialog() {
        let mut game = make_game();
        game.game_result.home.team_value = 1_040_000;
        game.game_result.away.team_value = 1_000_000;
        let mut step = StepBuyCards::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
        assert!(out.prompt.is_none());
        assert_eq!(gold(&out), (Some(0), Some(40_000)));
    }

    /// A gap of at least one card (the star tier: a hired star's cost is in Java's TV): the
    /// overdog is reported done at once, the underdog gets DialogBuyCardsParameter, and only its
    /// CLIENT_BUY_CARD(null) lets the step publish -- Java StepBuyCards.executeStep.
    #[test]
    fn underdog_with_card_gold_gets_the_dialog_then_declines() {
        let mut game = make_game();
        game.game_result.home.team_value = 1_000_000;
        game.game_result.away.team_value = 1_200_000;
        let mut step = StepBuyCards::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        match &out.prompt {
            Some(AgentPrompt::BuyCards { team_id, available_gold }) => {
                assert_eq!(team_id, &game.team_home.id);
                assert_eq!(*available_gold, 200_000);
            }
            other => panic!("expected BuyCards for home, got {other:?}"),
        }
        assert_eq!(gold(&out), (None, None), "nothing published while a team still chooses");
        assert!(step.reported_away && !step.reported_home, "overdog reported first");
        let out = step.handle_command(&Action::BuyCard { home: true, card_type: None }, &mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
        assert_eq!(gold(&out), (Some(200_000), Some(0)));
        assert!(step.reported_home);
    }

    /// The away underdog: the decline must come from the away coach.
    #[test]
    fn away_underdog_dialog_and_wrong_command_is_ignored() {
        let mut game = make_game();
        game.game_result.home.team_value = 1_300_000;
        game.game_result.away.team_value = 1_000_000;
        let mut step = StepBuyCards::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.prompt, Some(AgentPrompt::BuyCards { ref team_id, available_gold: 300_000 }) if *team_id == game.team_away.id));
        let out = step.handle_command(&Action::EndTurn, &mut game, &mut GameRng::new(0));
        assert!(out.prompt.is_none() && !matches!(out.action, StepAction::NextStep));
        let out = step.handle_command(&Action::BuyCard { home: false, card_type: None }, &mut game, &mut GameRng::new(0));
        assert_eq!(gold(&out), (Some(0), Some(300_000)));
    }
}
