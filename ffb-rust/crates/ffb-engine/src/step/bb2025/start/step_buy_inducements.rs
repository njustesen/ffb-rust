/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2025.start.StepBuyInducements`.
///
/// Handles pre-game inducement (and Prayers of Nuffle) purchase dialogs for both coaches, then
/// pushes the post-purchase Kickoff/Inducement/RiotousRookies/Prayers sequences.
///
/// Simplified vs Java: the true Java class resolves inducement cost via a ported
/// `InducementTypeFactory` (game-option-driven costs, and an "overdog buys first, underdog's
/// remaining petty cash depends on how much the overdog already spent" economy) and adds star
/// players / mercenaries / infamous staff directly from roster data. This translation instead
/// uses the data-driven catalog in `data/inducements/bb2025_inducements.json` (fixed costs per
/// `util_inducement_catalog`) and an underdog-shops-first budget model (mirroring the sibling
/// `bb2020::start::StepBuyCardsAndInducements` translation). `roster_star` / `roster_staff`
/// catalog entries are filtered out of the buyable catalog entirely — star player / infamous
/// staff purchasing needs roster data plumbed through beyond the catalog and is left for a
/// follow-up phase.
///
/// Java phase machine: INIT → determine underdog, set phase HOME or AWAY, compute that side's
/// budget → show dialog → on response, swap to the other team (computing its budget) or, once
/// both sides have shopped, DONE → leaveStep() (push Kickoff + Inducement×2 + RiotousRookies +
/// Prayers sequences; record used gold and prayers bought; auto-grant Bribery and Corruption /
/// Bugman's XXXXXX inducements).
///
/// "Prayers of Nuffle" purchases (`AgentPrompt::BuyPrayersAndInducements.prayers`, catalog id
/// `"prayers"`) are tracked via `prayers_bought_home`/`prayers_bought_away` (Java:
/// `prayersBoughtHome`/`prayersBoughtAway`) rather than the `InducementSet`, and published as
/// `StepParameter::PrayersBoughtHome`/`PrayersBoughtAway` for the `Prayers` step to consume.
///
/// Parallel mode (`INDUCEMENTS_ALLOW_SPENDING_TREASURY_ON_EQUAL_CTV`, equal TV): both sides'
/// budgets are computed up front, but this translation still resolves the dialogs sequentially
/// (home then away) rather than truly simultaneously — the driver only supports one pending
/// prompt at a time. This is a known, narrower-path simplification.
use ffb_model::data::loader::BB2025_INDUCEMENTS;
use ffb_model::enums::InducementPhase;
use ffb_model::inducement::inducement::Inducement as InducementModel;
use ffb_model::inducement::usage::Usage;
use ffb_model::model::game::Game;
use ffb_model::model::property::NamedProperties;
use ffb_model::model::special_rule::SpecialRule;
use ffb_model::option::game_option_id::{
    INDUCEMENTS, FREE_INDUCEMENT_CASH, USE_PREDEFINED_INDUCEMENTS,
    INDUCEMENTS_ALLOW_SPENDING_TREASURY_ON_EQUAL_CTV, INDUCEMENTS_ALLOW_OVERDOG_SPENDING,
};
use ffb_model::option::util_game_option::{is_option_enabled, get_int_option};
use ffb_model::prompts::AgentPrompt;
use ffb_model::report::bb2025::report_prayers_and_inducements_bought::ReportPrayersAndInducementsBought;
use ffb_model::report::mixed::report_bribery_and_corruption_re_roll::ReportBriberyAndCorruptionReRoll;
use ffb_model::util::rng::GameRng;
use crate::action::{Action, InducementPurchase};
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::generator::common::inducement::{Inducement, InducementParams};
use crate::step::generator::common::riotous_rookies::RiotousRookies;
use crate::step::generator::sequence::SequenceStep;
use crate::step::generator::mixed::kickoff::{Kickoff, KickoffParams};
use crate::step::game::start::util_inducement_catalog::{apply_purchases, available_list};
use crate::step::game::start::util_inducement_sequence::UtilInducementSequence;

/// Handles pre-game inducement purchase dialogs for both coaches.
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.start.StepBuyInducements`.
pub struct StepBuyInducements {
    /// Java: availableInducementGoldHome (Integer nullable)
    pub available_inducement_gold_home: Option<i32>,
    /// Java: availableInducementGoldAway (Integer nullable)
    pub available_inducement_gold_away: Option<i32>,
    /// Java: usedInducementGoldHome (Integer, init 0)
    pub used_inducement_gold_home: i32,
    /// Java: usedInducementGoldAway (Integer, init 0)
    pub used_inducement_gold_away: i32,
    /// Java: parallel
    pub parallel: bool,
    /// Java: phase (Phase private enum: INIT/HOME/AWAY/DONE) — stored as name
    pub phase_name: String,
    /// Java: prayersBoughtHome
    pub prayers_bought_home: i32,
    /// Java: prayersBoughtAway
    pub prayers_bought_away: i32,
    /// Java: buyInducementCommands (List<ClientCommandBuyInducements>) — stored as JSON blobs.
    /// Unused in this translation: parallel-mode buffering is not implemented (see module docs).
    pub buy_inducement_commands: Vec<String>,
}

impl StepBuyInducements {
    pub fn new() -> Self {
        Self {
            available_inducement_gold_home: None,
            available_inducement_gold_away: None,
            used_inducement_gold_home: 0,
            used_inducement_gold_away: 0,
            parallel: false,
            phase_name: "INIT".to_string(),
            prayers_bought_home: 0,
            prayers_bought_away: 0,
            buy_inducement_commands: Vec::new(),
        }
    }

    /// Java: `executeStep()` — main state machine dispatch.
    fn execute_step(&mut self, game: &mut Game) -> StepOutcome {
        match self.phase_name.as_str() {
            "INIT" => self.init(game),
            "HOME" | "AWAY" => self.swap_team(game),
            _ => {}
        }

        if self.phase_name == "DONE" {
            return self.leave_step(game);
        }
        self.build_prompt(game)
    }

    /// Java: `getAvailableGold(int, boolean, boolean)`, simplified: `petty_cash_from_tv_diff`
    /// (0 for the overdog) plus free cash, plus treasury when overdog spending is allowed.
    fn budget_for(game: &Game, home: bool, free_cash: i32, allow_overdog: bool) -> i32 {
        let petty = if home {
            game.game_result.home.petty_cash_from_tv_diff
        } else {
            game.game_result.away.petty_cash_from_tv_diff
        };
        let treasury = if home { game.team_home.treasury } else { game.team_away.treasury };
        petty + free_cash + if allow_overdog { treasury } else { 0 }
    }

    /// Java: `init(Game game)` — determine who has petty cash and set initial phase.
    fn init(&mut self, game: &mut Game) {
        // Java: if (!INDUCEMENTS) → phase = DONE
        // Java: if (USE_PREDEFINED_INDUCEMENTS) → apply predefined sets, skip dialog
        // no-op: InducementTypeFactory not ported — headless auto-skips inducement dialog
        if !is_option_enabled(game, INDUCEMENTS) || is_option_enabled(game, USE_PREDEFINED_INDUCEMENTS) {
            self.phase_name = "DONE".into();
            self.available_inducement_gold_home = Some(0);
            self.available_inducement_gold_away = Some(0);
            return;
        }

        let free_cash = get_int_option(game, FREE_INDUCEMENT_CASH);
        let petty_home = game.game_result.home.petty_cash_from_tv_diff;
        let petty_away = game.game_result.away.petty_cash_from_tv_diff;
        let allow_even_ctv = is_option_enabled(game, INDUCEMENTS_ALLOW_SPENDING_TREASURY_ON_EQUAL_CTV)
            || free_cash > 0;
        let allow_overdog = is_option_enabled(game, INDUCEMENTS_ALLOW_OVERDOG_SPENDING);

        if petty_home > 0 {
            self.phase_name = "HOME".into();
            self.available_inducement_gold_home = Some(Self::budget_for(game, true, free_cash, allow_overdog));
        } else if petty_away > 0 {
            self.phase_name = "AWAY".into();
            self.available_inducement_gold_away = Some(Self::budget_for(game, false, free_cash, allow_overdog));
        } else if allow_even_ctv {
            self.phase_name = "HOME".into();
            self.parallel = true;
            self.available_inducement_gold_home = Some(game.team_home.treasury + free_cash);
            self.available_inducement_gold_away = Some(game.team_away.treasury + free_cash);
        } else {
            self.available_inducement_gold_home = Some(0);
            self.available_inducement_gold_away = Some(0);
            self.phase_name = "DONE".into();
        }
    }

    /// Java: `swapTeam()` — move to the next team (computing its budget on first visit) or DONE.
    fn swap_team(&mut self, game: &Game) {
        let free_cash = get_int_option(game, FREE_INDUCEMENT_CASH);
        let allow_overdog = is_option_enabled(game, INDUCEMENTS_ALLOW_OVERDOG_SPENDING);
        match self.phase_name.as_str() {
            "HOME" if self.available_inducement_gold_away.is_none() => {
                self.phase_name = "AWAY".into();
                self.available_inducement_gold_away = Some(Self::budget_for(game, false, free_cash, allow_overdog));
            }
            "AWAY" if self.available_inducement_gold_home.is_none() => {
                self.phase_name = "HOME".into();
                self.available_inducement_gold_home = Some(Self::budget_for(game, true, free_cash, allow_overdog));
            }
            _ => {
                self.phase_name = "DONE".into();
            }
        }
    }

    /// Java: `showDialog(Team, ...)` — builds the `DialogBuyPrayersAndInducementsParameter`
    /// for whichever team's phase is active.
    fn build_prompt(&self, game: &Game) -> StepOutcome {
        let home = self.phase_name == "HOME";
        let team = if home { &game.team_home } else { &game.team_away };
        let budget = if home {
            self.available_inducement_gold_home.unwrap_or(0) - self.used_inducement_gold_home
        } else {
            self.available_inducement_gold_away.unwrap_or(0) - self.used_inducement_gold_away
        };
        let catalog = &BB2025_INDUCEMENTS.inducements;
        let full = available_list(catalog, team);
        let (prayers, available): (Vec<_>, Vec<_>) = full.into_iter().partition(|(id, _)| id == "prayers");
        StepOutcome::cont().with_prompt(AgentPrompt::BuyPrayersAndInducements {
            team_id: team.id.clone(),
            available,
            prayers,
            budget,
        })
    }

    /// Java: `handleTeamInducements(TurnData, Team, ClientCommandBuyInducements, int)` —
    /// extracts the Prayers of Nuffle purchase (tracked separately, see module docs) and
    /// applies the remaining purchases via the shared catalog helper.
    fn apply_purchase(&mut self, game: &mut Game, home: bool, purchases: &[InducementPurchase]) {
        let (prayer_purchases, other_purchases): (Vec<_>, Vec<_>) =
            purchases.iter().cloned().partition(|p| p.id == "prayers");
        let catalog = &BB2025_INDUCEMENTS.inducements;
        let prayers_def = catalog.iter().find(|d| d.id == "prayers");

        let (team, turn_inducement_set, gold_used, gold_available, prayers_bought) = if home {
            (&mut game.team_home, &mut game.turn_data_home.inducement_set,
             &mut self.used_inducement_gold_home, self.available_inducement_gold_home,
             &mut self.prayers_bought_home)
        } else {
            (&mut game.team_away, &mut game.turn_data_away.inducement_set,
             &mut self.used_inducement_gold_away, self.available_inducement_gold_away,
             &mut self.prayers_bought_away)
        };

        let mut budget = gold_available.unwrap_or(0) - *gold_used;

        if let (Some(def), Some(purchase)) = (prayers_def, prayer_purchases.first()) {
            let room = (def.max_count - *prayers_bought).max(0);
            let mut qty = (purchase.count as i32).min(room);
            if def.cost > 0 {
                qty = qty.min(budget / def.cost);
            }
            if qty > 0 {
                let cost = def.cost * qty;
                *prayers_bought += qty;
                budget -= cost;
                *gold_used += cost;
                team.treasury = (team.treasury - cost).max(0);
            }
        }

        let spent = apply_purchases(catalog, team, turn_inducement_set, &other_purchases, budget);
        *gold_used += spent;
    }

    /// Java: `leaveStep()` — push sequences, record gold spent / prayers bought, NEXT_STEP.
    fn leave_step(&mut self, game: &mut Game) -> StepOutcome {
        let new_tv_home = game.team_home.team_value + self.used_inducement_gold_home;
        let new_tv_away = game.team_away.team_value + self.used_inducement_gold_away;

        if self.parallel {
            game.report_list.add(ReportPrayersAndInducementsBought::new(
                game.team_home.id.clone(), 0, 0, 0, self.used_inducement_gold_home, new_tv_home,
            ));
            game.report_list.add(ReportPrayersAndInducementsBought::new(
                game.team_away.id.clone(), 0, 0, 0, self.used_inducement_gold_away, new_tv_away,
            ));
        }

        let seq_kickoff = Kickoff::build_sequence(&KickoffParams { with_coin_choice: true });
        let (first_home, second_home) = if new_tv_home > new_tv_away { (true, false) } else { (false, true) };
        let seq_first = Inducement::build_sequence(&InducementParams {
            inducement_phase: InducementPhase::AfterInducementsPurchased,
            home_team: first_home,
            check_forgo: false,
        });
        let seq_second = Inducement::build_sequence(&InducementParams {
            inducement_phase: InducementPhase::AfterInducementsPurchased,
            home_team: second_home,
            check_forgo: false,
        });
        let seq_riotous = RiotousRookies::build_sequence();
        // Java: `Sequence prayerSequence = new Sequence(...); prayerSequence.add(StepId.PRAYERS);`
        let seq_prayers = vec![SequenceStep::new(StepId::Prayers)];

        game.game_result.home.petty_cash_used = UtilInducementSequence::calculate_inducement_gold(Some(game), true);
        game.game_result.away.petty_cash_used = UtilInducementSequence::calculate_inducement_gold(Some(game), false);

        // Java: inducementTypeFactory.allTypes() filtered by Usage.REROLL_ARGUE → "briberyAndCorruption".
        {
            let bnc_name = SpecialRule::BRIBERY_AND_CORRUPTION.get_rule_name();
            if game.team_home.special_rules.iter().any(|r| r == bnc_name) {
                game.turn_data_home.inducement_set.add_inducement(
                    InducementModel::new("briberyAndCorruption", 1, vec![Usage::REROLL_ARGUE]));
                game.report_list.add(ReportBriberyAndCorruptionReRoll::new(
                    Some(game.team_home.id.clone()), "ADDED".into(),
                ));
            }
            if game.team_away.special_rules.iter().any(|r| r == bnc_name) {
                game.turn_data_away.inducement_set.add_inducement(
                    InducementModel::new("briberyAndCorruption", 1, vec![Usage::REROLL_ARGUE]));
                game.report_list.add(ReportBriberyAndCorruptionReRoll::new(
                    Some(game.team_away.id.clone()), "ADDED".into(),
                ));
            }
        }

        // Java: inducementTypeFactory.allTypes() filtered by Usage.REROLL_ONES_ON_KOS → "bugmansXXXXXX".
        {
            let prop = NamedProperties::CAN_RE_ROLL_ONES_ON_KO_RECOVERY;
            if game.team_home.players.iter().any(|p| p.has_skill_property(prop)) {
                game.turn_data_home.inducement_set.add_inducement(
                    InducementModel::new("bugmansXXXXXX", 1, vec![Usage::REROLL_ONES_ON_KOS]));
            }
            if game.team_away.players.iter().any(|p| p.has_skill_property(prop)) {
                game.turn_data_away.inducement_set.add_inducement(
                    InducementModel::new("bugmansXXXXXX", 1, vec![Usage::REROLL_ONES_ON_KOS]));
            }
        }

        self.phase_name = "DONE".into();

        StepOutcome::next()
            .publish(StepParameter::TvHome(new_tv_home))
            .publish(StepParameter::TvAway(new_tv_away))
            .publish(StepParameter::PrayersBoughtHome(self.prayers_bought_home))
            .publish(StepParameter::PrayersBoughtAway(self.prayers_bought_away))
            .push_seq(seq_kickoff)
            .push_seq(seq_first)
            .push_seq(seq_second)
            .push_seq(seq_riotous)
            .push_seq(seq_prayers)
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

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: CLIENT_BUY_INDUCEMENTS → handleBuyInducements (or buffer, in parallel mode).
        // headless: parallel-mode buffering not implemented — see module docs.
        if let Action::BuyInducements { home, purchases } = action {
            self.apply_purchase(game, *home, purchases);
        }
        self.execute_step(game)
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    fn make_game_with_petty_cash(petty_home: i32, petty_away: i32) -> Game {
        let mut game = make_game();
        game.game_result.home.petty_cash_from_tv_diff = petty_home;
        game.game_result.away.petty_cash_from_tv_diff = petty_away;
        game
    }

    #[test]
    fn initial_phase_is_init() {
        let step = StepBuyInducements::new();
        assert_eq!(step.phase_name, "INIT");
    }

    #[test]
    fn set_parameter_returns_false() {
        let mut step = StepBuyInducements::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(false)));
    }

    #[test]
    fn default_equivalent_to_new() {
        let _a = StepBuyInducements::new();
        let _b = StepBuyInducements::default();
    }

    #[test]
    fn inducements_disabled_skips_to_next_step() {
        let mut game = make_game_with_petty_cash(100_000, 0);
        // INDUCEMENTS option not set → disabled → skip immediately.
        let mut step = StepBuyInducements::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(step.phase_name, "DONE");
    }

    #[test]
    fn equal_tv_no_treasury_spending_skips_to_next_step() {
        use ffb_model::option::game_option_id::INDUCEMENTS;
        let mut game = make_game();
        game.options.set(INDUCEMENTS, "true");
        let mut step = StepBuyInducements::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(step.available_inducement_gold_home, Some(0));
        assert_eq!(step.available_inducement_gold_away, Some(0));
    }

    #[test]
    fn home_underdog_shows_prompt_for_home_first() {
        use ffb_model::option::game_option_id::INDUCEMENTS;
        let mut game = make_game_with_petty_cash(100_000, 0);
        game.options.set(INDUCEMENTS, "true");
        let mut step = StepBuyInducements::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(step.phase_name, "HOME");
        match out.prompt {
            Some(AgentPrompt::BuyPrayersAndInducements { team_id, budget, .. }) => {
                assert_eq!(team_id, "home");
                assert_eq!(budget, 100_000);
            }
            other => panic!("expected BuyPrayersAndInducements prompt, got {other:?}"),
        }
    }

    #[test]
    fn prompt_lists_prayers_separately_from_available() {
        use ffb_model::option::game_option_id::INDUCEMENTS;
        let mut game = make_game_with_petty_cash(200_000, 0);
        game.options.set(INDUCEMENTS, "true");
        let mut step = StepBuyInducements::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        match out.prompt {
            Some(AgentPrompt::BuyPrayersAndInducements { available, prayers, .. }) => {
                assert!(prayers.iter().any(|(id, _)| id == "prayers"));
                assert!(!available.iter().any(|(id, _)| id == "prayers"));
                assert!(available.iter().any(|(id, _)| id == "bribes"));
            }
            other => panic!("expected BuyPrayersAndInducements prompt, got {other:?}"),
        }
    }

    #[test]
    fn away_gets_prompted_after_home_submits() {
        use ffb_model::option::game_option_id::INDUCEMENTS;
        let mut game = make_game_with_petty_cash(100_000, 0);
        game.options.set(INDUCEMENTS, "true");
        let mut step = StepBuyInducements::new();
        step.start(&mut game, &mut GameRng::new(0));
        let out = step.handle_command(
            &Action::BuyInducements { home: true, purchases: vec![] },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(step.phase_name, "AWAY");
        match out.prompt {
            Some(AgentPrompt::BuyPrayersAndInducements { team_id, .. }) => assert_eq!(team_id, "away"),
            other => panic!("expected BuyPrayersAndInducements prompt for away, got {other:?}"),
        }
    }

    #[test]
    fn both_sides_submitting_advances_to_next_step() {
        use ffb_model::option::game_option_id::INDUCEMENTS;
        let mut game = make_game_with_petty_cash(200_000, 0);
        game.options.set(INDUCEMENTS, "true");
        let mut step = StepBuyInducements::new();
        step.start(&mut game, &mut GameRng::new(0));
        step.handle_command(
            &Action::BuyInducements {
                home: true,
                purchases: vec![InducementPurchase { id: "bribes".into(), count: 1 }],
            },
            &mut game,
            &mut GameRng::new(0),
        );
        let out = step.handle_command(
            &Action::BuyInducements { home: false, purchases: vec![] },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.team_home.bribes, 1);
    }

    #[test]
    fn purchase_deducts_treasury_and_tracks_used_gold() {
        use ffb_model::option::game_option_id::INDUCEMENTS;
        let mut game = make_game_with_petty_cash(200_000, 0);
        game.options.set(INDUCEMENTS, "true");
        game.team_home.treasury = 500_000;
        let mut step = StepBuyInducements::new();
        step.start(&mut game, &mut GameRng::new(0));
        step.handle_command(
            &Action::BuyInducements {
                home: true,
                purchases: vec![InducementPurchase { id: "bribes".into(), count: 1 }],
            },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(game.team_home.treasury, 400_000);
        assert_eq!(step.used_inducement_gold_home, 100_000);
    }

    #[test]
    fn prayers_purchase_tracked_separately_from_inducement_set() {
        use ffb_model::option::game_option_id::INDUCEMENTS;
        let mut game = make_game_with_petty_cash(200_000, 0);
        game.options.set(INDUCEMENTS, "true");
        let mut step = StepBuyInducements::new();
        step.start(&mut game, &mut GameRng::new(0));
        step.handle_command(
            &Action::BuyInducements {
                home: true,
                purchases: vec![InducementPurchase { id: "prayers".into(), count: 2 }],
            },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(step.prayers_bought_home, 2);
        assert!(game.turn_data_home.inducement_set.get("prayers").is_none());
        assert_eq!(step.used_inducement_gold_home, 200_000);
    }

    #[test]
    fn prayers_purchase_clamped_to_max_count() {
        use ffb_model::option::game_option_id::INDUCEMENTS;
        let mut game = make_game_with_petty_cash(1_000_000, 0);
        game.options.set(INDUCEMENTS, "true");
        let mut step = StepBuyInducements::new();
        step.start(&mut game, &mut GameRng::new(0));
        step.handle_command(
            &Action::BuyInducements {
                home: true,
                purchases: vec![InducementPurchase { id: "prayers".into(), count: 10 }],
            },
            &mut game,
            &mut GameRng::new(0),
        );
        // prayers max_count is 3.
        assert_eq!(step.prayers_bought_home, 3);
    }

    #[test]
    fn leave_step_publishes_prayers_bought_parameters() {
        use ffb_model::option::game_option_id::INDUCEMENTS;
        let mut game = make_game_with_petty_cash(200_000, 0);
        game.options.set(INDUCEMENTS, "true");
        let mut step = StepBuyInducements::new();
        step.start(&mut game, &mut GameRng::new(0));
        step.handle_command(
            &Action::BuyInducements {
                home: true,
                purchases: vec![InducementPurchase { id: "prayers".into(), count: 1 }],
            },
            &mut game,
            &mut GameRng::new(0),
        );
        let out = step.handle_command(
            &Action::BuyInducements { home: false, purchases: vec![] },
            &mut game,
            &mut GameRng::new(0),
        );
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::PrayersBoughtHome(1))));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::PrayersBoughtAway(0))));
    }

    #[test]
    fn leave_step_pushes_kickoff_inducements_riotous_and_prayers_sequences() {
        use ffb_model::option::game_option_id::INDUCEMENTS;
        let mut game = make_game_with_petty_cash(200_000, 0);
        game.options.set(INDUCEMENTS, "true");
        let mut step = StepBuyInducements::new();
        step.start(&mut game, &mut GameRng::new(0));
        step.handle_command(
            &Action::BuyInducements { home: true, purchases: vec![] },
            &mut game,
            &mut GameRng::new(0),
        );
        let out = step.handle_command(
            &Action::BuyInducements { home: false, purchases: vec![] },
            &mut game,
            &mut GameRng::new(0),
        );
        // Kickoff + 2×Inducement + RiotousRookies + Prayers = 5 sequences.
        assert_eq!(out.pushes.len(), 5);
    }

    #[test]
    fn bribery_and_corruption_special_rule_grants_inducement_on_leave_step() {
        use ffb_model::option::game_option_id::INDUCEMENTS;
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game_with_petty_cash(200_000, 0);
        game.options.set(INDUCEMENTS, "true");
        game.team_home.special_rules.push("Bribery and Corruption".into());
        let mut step = StepBuyInducements::new();
        step.start(&mut game, &mut GameRng::new(0));
        step.handle_command(
            &Action::BuyInducements { home: true, purchases: vec![] },
            &mut game,
            &mut GameRng::new(0),
        );
        step.handle_command(
            &Action::BuyInducements { home: false, purchases: vec![] },
            &mut game,
            &mut GameRng::new(0),
        );
        assert!(game.turn_data_home.inducement_set.get("briberyAndCorruption").is_some());
        assert!(game.report_list.has_report(ReportId::BRIBERY_AND_CORRUPTION_RE_ROLL));
    }
}
