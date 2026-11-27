/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2025.start.StepBuyInducements`.
///
/// Handles pre-game inducement (and Prayers of Nuffle) purchase dialogs for both coaches, then
/// pushes the post-purchase Kickoff/Inducement/RiotousRookies/Prayers sequences.
///
/// Costs, maxima and availability come from the bb2025 `InducementTypeFactory` types (§Q2.8):
/// `getActualCostId(team)` options, `availability(team, options)` with the per-type overrides
/// (`InducementTypeEntry`). `handleTeamInducements` adds the bought set to the turn's
/// InducementSet and charges `inducementCosts`, as Java does. Not modelled yet: star players,
/// mercenaries and infamous staff (position choices; the agents never buy them).
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
use ffb_model::enums::InducementPhase;
use ffb_model::events::GameEvent;
use ffb_model::inducement::inducement::Inducement as InducementModel;
use ffb_model::inducement::usage::Usage;
use ffb_model::model::game::Game;
use ffb_model::model::property::NamedProperties;
use ffb_model::model::special_rule::SpecialRule;
use ffb_model::option::game_option_id::{
    INDUCEMENTS, FREE_INDUCEMENT_CASH, USE_PREDEFINED_INDUCEMENTS,
    INDUCEMENTS_ALLOW_SPENDING_TREASURY_ON_EQUAL_CTV, INDUCEMENTS_ALLOW_OVERDOG_SPENDING,
    INDUCEMENTS_ALWAYS_USE_TREASURY,
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
use crate::step::game::start::util_minimum_inducement_cost::minimum_inducement_cost;

/// Java `game.getFactory(INDUCEMENT_TYPE).allTypes()` for BB2025: the base collection plus the
/// bb2025 sub-types, sorted by name.
fn bb2025_inducement_types() -> Vec<ffb_model::inducement::inducement_collection::InducementTypeEntry> {
    ffb_model::factory::inducement_type_factory::all_types_in(ffb_model::enums::Rules::Bb2025)
}

/// Java: `StepBuyInducements.MAX_UNDERDOG_ALLOWANCE` — the most treasury an underdog may add to
/// its petty cash.
pub const MAX_UNDERDOG_ALLOWANCE: i32 = 50_000;

/// Java `StepBuyInducements.leaveStep`: grant the special-rule / skill-derived inducements that
/// exist regardless of any purchase — Bribery and Corruption (special rule → REROLL_ARGUE) and
/// Bugman's XXXXXX (a canReRollOnesOnKORecovery player → REROLL_ONES_ON_KOS). Dice-free; a pure
/// function of the drafted rosters. Extracted so the parity start sequence — which omits the
/// petty-cash/shopping steps (see `sequences::start_game_sequence`) — can still apply the exact
/// same grant Java's leaveStep does (dwarf seed 2: the Deathroller's argue natural-1 must be
/// re-rollable via the team's Bribery and Corruption).
pub fn grant_special_rule_inducements(game: &mut Game) {
    // Java: allTypes filtered by Usage.REROLL_ARGUE → "briberyAndCorruption".
    let bnc_name = SpecialRule::BRIBERY_AND_CORRUPTION.get_rule_name();
    if game.team_home.special_rules.iter().any(|r| r == bnc_name) {
        game.turn_data_home.inducement_set.add_inducement(
            InducementModel::new("briberyAndCorruption", 1, vec![Usage::REROLL_ARGUE]));
        game.report_list.add(ReportBriberyAndCorruptionReRoll::new(
            Some(game.team_home.id.clone()), "ADDED".into()));
    }
    if game.team_away.special_rules.iter().any(|r| r == bnc_name) {
        game.turn_data_away.inducement_set.add_inducement(
            InducementModel::new("briberyAndCorruption", 1, vec![Usage::REROLL_ARGUE]));
        game.report_list.add(ReportBriberyAndCorruptionReRoll::new(
            Some(game.team_away.id.clone()), "ADDED".into()));
    }
    // Java: allTypes filtered by Usage.REROLL_ONES_ON_KOS → "bugmansXXXXXX".
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
    /// Java: transient pettyCash (the dialog's petty-cash share, set by getAvailableGold)
    pub petty_cash: i32,
    /// Java: transient treasury (the dialog's treasury share, set by getAvailableGold)
    pub treasury: i32,
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
    /// The dialog `showDialog` opened and that is still waiting for its answer:
    /// (home?, availableGold). Java hands it to `UtilServerDialog.showDialog`; here it becomes
    /// the step's prompt.
    dialog: Option<(bool, i32, bool)>,
}

impl StepBuyInducements {
    pub fn new() -> Self {
        Self {
            available_inducement_gold_home: None,
            available_inducement_gold_away: None,
            used_inducement_gold_home: 0,
            used_inducement_gold_away: 0,
            petty_cash: 0,
            treasury: 0,
            parallel: false,
            phase_name: "INIT".to_string(),
            prayers_bought_home: 0,
            prayers_bought_away: 0,
            buy_inducement_commands: Vec::new(),
            dialog: None,
        }
    }

    /// Java: `executeStep()` — main state machine dispatch.
    fn execute_step(&mut self, game: &mut Game) -> StepOutcome {
        self.dialog = None;
        match self.phase_name.as_str() {
            "INIT" => self.init(game),
            "HOME" | "AWAY" => self.swap_team(game),
            _ => {}
        }

        if self.phase_name == "DONE" {
            return self.leave_step(game);
        }
        if let Some((home, available_gold, uses_treasury)) = self.dialog {
            // Java showDialog(...): UtilServerDialog.showDialog(getGameState(), new
            // DialogBuyPrayersAndInducementsParameter(team.getId(), availableGold, usesUnlimitedTreasury,
            // pettyCash, treasury), false)
            let team_id = if home { game.team_home.id.clone() } else { game.team_away.id.clone() };
            crate::util::util_server_dialog::UtilServerDialog::show_dialog(
                game,
                ffb_model::dialog::dialog_parameter_factory::DialogParameter::BuyPrayersAndInducements(
                    ffb_model::dialog::dialog_buy_prayers_and_inducements_parameter::DialogBuyPrayersAndInducementsParameter {
                        team_id: Some(team_id),
                        available_gold,
                        petty_cash: self.petty_cash,
                        treasury: self.treasury,
                        uses_treasury,
                    },
                ),
                false,
            );
        }
        self.build_prompt(game)
    }

    /// Java: `init(Game game)`. The team that does NOT receive petty cash (the overdog) is asked
    /// first, from its treasury when overdog spending is allowed; with equal team values and
    /// no reason to spend, the phase is DONE without any dialog.
    fn init(&mut self, game: &mut Game) {
        if !is_option_enabled(game, INDUCEMENTS) {
            self.phase_name = "DONE".into();
            return;
        }
        // Java: USE_PREDEFINED_INDUCEMENTS applies the teams' predefined sets and goes DONE.
        // no-op: predefined sets are not modelled (see module docs); DONE without a dialog.
        if is_option_enabled(game, USE_PREDEFINED_INDUCEMENTS) {
            self.phase_name = "DONE".into();
            return;
        }

        let free_cash = get_int_option(game, FREE_INDUCEMENT_CASH);
        let allow_inducements_on_even_ctv =
            is_option_enabled(game, INDUCEMENTS_ALLOW_SPENDING_TREASURY_ON_EQUAL_CTV) || free_cash > 0;
        let mut same_tv = false;

        let overdog_home = if game.game_result.home.petty_cash_from_tv_diff > 0 {
            self.phase_name = "AWAY".into();
            false
        } else if game.game_result.away.petty_cash_from_tv_diff > 0 {
            self.phase_name = "HOME".into();
            true
        } else if allow_inducements_on_even_ctv {
            self.phase_name = "HOME".into();
            self.parallel = true;
            same_tv = true;
            true
        } else {
            self.phase_name = "DONE".into();
            self.available_inducement_gold_home = Some(0);
            self.available_inducement_gold_away = Some(0);
            return;
        };

        let allow_overdog_spending = is_option_enabled(game, INDUCEMENTS_ALLOW_OVERDOG_SPENDING);
        if !self.show_dialog(game, overdog_home, free_cash, true, allow_overdog_spending || same_tv) {
            self.swap_team(game);
        }
    }

    /// Java: `getAvailableGold(int freeCash, boolean useUnlimitedTreasury, boolean allowSpending)`
    /// for the team whose phase is active. Sets the transient `petty_cash` / `treasury` split the
    /// dialog carries, and records the team's available gold.
    fn get_available_gold(&mut self, game: &Game, free_cash: i32, use_unlimited_treasury: bool,
                          allow_spending: bool) -> i32 {
        let home = self.phase_name == "HOME";
        let (team_treasury, petty_from_tv_diff, other_used) = if home {
            (game.team_home.treasury, game.game_result.home.petty_cash_from_tv_diff, self.used_inducement_gold_away)
        } else {
            (game.team_away.treasury, game.game_result.away.petty_cash_from_tv_diff, self.used_inducement_gold_home)
        };
        let available = if allow_spending {
            if use_unlimited_treasury {
                self.treasury = free_cash + team_treasury;
                self.petty_cash = 0;
                self.treasury
            } else {
                self.petty_cash = (other_used - free_cash).max(0) + petty_from_tv_diff + free_cash;
                self.treasury = MAX_UNDERDOG_ALLOWANCE.min(team_treasury);
                self.petty_cash + self.treasury
            }
        } else {
            self.petty_cash = 0;
            self.treasury = 0;
            0
        };
        if home {
            self.available_inducement_gold_home = Some(available);
        } else {
            self.available_inducement_gold_away = Some(available);
        }
        available
    }

    /// Java: `showDialog(Team, freeCash, usesUnlimitedTreasury, allowSpending)` — computes the
    /// team's gold and opens its dialog when it can afford the cheapest inducement.
    fn show_dialog(&mut self, game: &Game, home: bool, free_cash: i32, uses_unlimited_treasury: bool,
                   allow_spending: bool) -> bool {
        let available_gold = self.get_available_gold(game, free_cash, uses_unlimited_treasury, allow_spending);
        let team = if home { &game.team_home } else { &game.team_away };
        let can_buy_inducements = minimum_inducement_cost(game, team) <= available_gold;
        if can_buy_inducements {
            self.dialog = Some((home, available_gold, uses_unlimited_treasury));
            return true;
        }
        false
    }

    /// Java: `swapTeam()` — move to the team that has not been visited yet and show its dialog,
    /// or go DONE once both have been (or the second team cannot afford anything).
    fn swap_team(&mut self, game: &Game) {
        let home = if self.phase_name == "HOME" && self.available_inducement_gold_away.is_none() {
            self.phase_name = "AWAY".into();
            false
        } else if self.phase_name == "AWAY" && self.available_inducement_gold_home.is_none() {
            self.phase_name = "HOME".into();
            true
        } else {
            self.phase_name = "DONE".into();
            return;
        };
        let free_cash = get_int_option(game, FREE_INDUCEMENT_CASH);
        let unlimited = self.parallel || is_option_enabled(game, INDUCEMENTS_ALWAYS_USE_TREASURY);
        if !self.show_dialog(game, home, free_cash, unlimited, true) {
            self.phase_name = "DONE".into();
        }
    }

    /// The open `DialogBuyPrayersAndInducementsParameter` as a prompt. Its budget is the
    /// dialog's `availableGold`.
    fn build_prompt(&self, game: &Game) -> StepOutcome {
        let (home, budget) = match self.dialog {
            Some((home, budget, _)) => (home, budget),
            // Java never leaves executeStep in HOME/AWAY without an open dialog.
            None => return StepOutcome::cont(),
        };
        let team = if home { &game.team_home } else { &game.team_away };
        // §Q2.8: the types the Java dialog lets the coach buy -- every type of the bb2025
        // InducementTypeFactory with a cost option (`getActualCostId(team)`), availability > 0 and
        // not `card`, priced by that option (as `inducementCosts` charges it). Stars, mercenaries and
        // infamous staff are position choices, not modelled here.
        let full: Vec<(String, i32)> = bb2025_inducement_types().into_iter()
            .filter(|t| t.id != "card" && t.availability(team, &game.options) > 0)
            .filter_map(|t| t.actual_cost_id(team).map(|cost_id| (t.id.clone(), get_int_option(game, cost_id))))
            .collect();
        let (prayers, available): (Vec<_>, Vec<_>) = full.into_iter().partition(|(id, _)| id == "prayers");
        StepOutcome::cont().with_prompt(AgentPrompt::BuyPrayersAndInducements {
            team_id: team.id.clone(),
            available,
            prayers,
            budget,
        })
    }

    /// Java: `generateReport(Team, gold, newTv, boughtPrayers)`.
    fn generate_report(game: &Game, home: bool, gold: i32, new_tv: i32, bought_prayers: i32)
        -> ReportPrayersAndInducementsBought
    {
        let (team_id, inducement_set) = if home {
            (game.team_home.id.clone(), &game.turn_data_home.inducement_set)
        } else {
            (game.team_away.id.clone(), &game.turn_data_away.inducement_set)
        };
        let mut nr_of_inducements = bought_prayers;
        let mut nr_of_stars = 0;
        let mut nr_of_mercenaries = 0;
        for inducement in inducement_set.get_inducements() {
            if inducement.has_usage(Usage::STAR) {
                nr_of_stars = inducement.get_value();
            } else if inducement.has_usage(Usage::LONER) {
                nr_of_mercenaries = inducement.get_value();
            } else {
                nr_of_inducements += inducement.get_value();
            }
        }
        ReportPrayersAndInducementsBought::new(team_id, nr_of_inducements, nr_of_stars, nr_of_mercenaries, gold, new_tv)
    }

    /// Java: `handleTeamInducements(TurnData, Team, ClientCommandBuyInducements, int)` —
    /// extracts the Prayers of Nuffle purchase (tracked separately, see module docs) and
    /// applies the remaining purchases via the shared catalog helper. Returns a
    /// `GameEvent::BuyInducement` per inducement id actually purchased (including "prayers"),
    /// so coverage tooling can see that a purchase happened.
    fn apply_purchase(&mut self, game: &mut Game, home: bool, purchases: &[InducementPurchase]) -> Vec<GameEvent> {
        // Java `handleTeamInducements`: the command's InducementSet (one Inducement per type, the
        // last one for a repeated type, as `addInducement` keys by type).
        let types = bb2025_inducement_types();
        let mut inducement_set = ffb_model::model::inducement_set::InducementSet::new();
        for purchase in purchases {
            if let Some(t) = types.iter().find(|t| t.id == purchase.id) {
                inducement_set.add_inducement(InducementModel::new(t.id.clone(), purchase.count as i32, t.usages.clone()));
            }
        }
        let team = if home { &game.team_home } else { &game.team_away };
        let team_id = team.id.clone();
        // Java `inducementCosts(team, set)`: value × the `getActualCostId(team)` option, for every
        // inducement whose type has a cost id (prayers included -- they are removed only after).
        let inducement_cost: i32 = inducement_set.get_inducements().iter()
            .filter_map(|i| types.iter().find(|t| t.id == i.type_id)
                .and_then(|t| t.actual_cost_id(team))
                .map(|cost_id| i.get_value() * get_int_option(game, cost_id)))
            .sum();
        // Java: the GAME_MODIFICATION type (prayers) leaves the set; its value is the prayers bought.
        if let Some(prayers) = inducement_set.get("prayers") {
            inducement_set.remove_inducement("prayers");
            if home { self.prayers_bought_home = prayers.get_value(); } else { self.prayers_bought_away = prayers.get_value(); }
        }
        let mut events: Vec<GameEvent> = inducement_set.get_inducements().iter()
            .map(|i| GameEvent::BuyInducement {
                team_id: team_id.clone(), inducement_id: i.type_id.clone(), count: i.get_value() as u32,
            })
            .collect();
        events.sort_by(|a, b| format!("{a:?}").cmp(&format!("{b:?}")));
        let prayers_bought = if home { self.prayers_bought_home } else { self.prayers_bought_away };
        if prayers_bought > 0 {
            events.push(GameEvent::BuyInducement {
                team_id: team_id.clone(), inducement_id: "prayers".to_string(), count: prayers_bought as u32,
            });
        }
        // Java: `turnData.getInducementSet().add(inducementSet)`; no stars, staff or mercenaries.
        if home {
            game.turn_data_home.inducement_set.add(&inducement_set);
            self.used_inducement_gold_home = inducement_cost;
        } else {
            game.turn_data_away.inducement_set.add(&inducement_set);
            self.used_inducement_gold_away = inducement_cost;
        }
        // Java throws when a team spends more than its available gold; the agents never do.
        let available = if home { self.available_inducement_gold_home } else { self.available_inducement_gold_away };
        debug_assert!(inducement_cost <= available.unwrap_or(0), "{team_id} overspent inducement gold");
        events
    }

    /// Java: `leaveStep()` — push sequences, record gold spent / prayers bought, NEXT_STEP.
    fn leave_step(&mut self, game: &mut Game) -> StepOutcome {
        let new_tv_home = game.team_home.team_value + self.used_inducement_gold_home;
        let new_tv_away = game.team_away.team_value + self.used_inducement_gold_away;

        if self.parallel {
            let report = Self::generate_report(game, true, self.used_inducement_gold_home, new_tv_home,
                self.prayers_bought_home);
            game.report_list.add(report);
            let report = Self::generate_report(game, false, self.used_inducement_gold_away, new_tv_away,
                self.prayers_bought_away);
            game.report_list.add(report);
        }

        // Java: `Kickoff.pushSequence(new Kickoff.SequenceParams(gameState, true))` — the
        // game-opening kickoff with the coin toss, in the certified form (see
        // `sequences::opening_kickoff_sequence_for`).
        let seq_kickoff = crate::step::sequences::opening_kickoff_sequence_for(game.rules);
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

        // Java: treasury / petty-cash bookkeeping. A team without petty cash (or when the treasury
        // is always used) spent its treasury; the underdog splits its spend via
        // setUnderDogCashValues.
        let always_use_treasury = is_option_enabled(game, INDUCEMENTS_ALWAYS_USE_TREASURY);
        let free_cash = get_int_option(game, FREE_INDUCEMENT_CASH);
        if game.game_result.home.petty_cash_from_tv_diff == 0 || always_use_treasury {
            game.game_result.home.treasury_spent_on_inducements = (self.used_inducement_gold_home - free_cash).max(0);
        } else {
            self.set_under_dog_cash_values(game, true, free_cash);
        }
        if game.game_result.away.petty_cash_from_tv_diff == 0 || always_use_treasury {
            game.game_result.away.treasury_spent_on_inducements = (self.used_inducement_gold_away - free_cash).max(0);
        } else {
            self.set_under_dog_cash_values(game, false, free_cash);
        }

        grant_special_rule_inducements(game);

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

    /// Java: `setUnderDogCashValues(TeamResult, usedInducementGold, availableInducementGold,
    /// freeCash)`. Uses the transient `treasury` share of the last dialog shown, as Java does.
    fn set_under_dog_cash_values(&self, game: &mut Game, home: bool, free_cash: i32) {
        let (used, available) = if home {
            (self.used_inducement_gold_home, self.available_inducement_gold_home.unwrap_or(0))
        } else {
            (self.used_inducement_gold_away, self.available_inducement_gold_away.unwrap_or(0))
        };
        let unspent = available - used;
        let unspent_allowance = unspent.min(self.treasury);
        let treasury_spent = self.treasury - unspent_allowance;
        let used_petty_cash = used - treasury_spent - free_cash;
        let result = if home { &mut game.game_result.home } else { &mut game.game_result.away };
        result.treasury_spent_on_inducements = treasury_spent;
        result.petty_cash_used = used_petty_cash.min(available - free_cash);
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
        // Java: CLIENT_BUY_INDUCEMENTS → handleBuyInducements, then (not parallel) report this
        // team's purchase straight away: `addReport(generateReport(team, used, newTv, prayers))`.
        // headless: parallel-mode buffering not implemented — see module docs.
        let mut events = Vec::new();
        // Java: any other command is UNHANDLED -- executeStep does not run again.
        if !matches!(action, Action::BuyInducements { .. }) {
            return self.build_prompt(game);
        }
        if let Action::BuyInducements { home, purchases } = action {
            events = self.apply_purchase(game, *home, purchases);
            if !self.parallel {
                let (used, prayers, team_value) = if *home {
                    (self.used_inducement_gold_home, self.prayers_bought_home, game.team_home.team_value)
                } else {
                    (self.used_inducement_gold_away, self.prayers_bought_away, game.team_away.team_value)
                };
                let report = Self::generate_report(game, *home, used, team_value + used, prayers);
                game.report_list.add(report);
            }
        }
        let mut out = self.execute_step(game);
        for ev in events { out = out.with_event(ev); }
        out
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
    fn inducements_disabled_skips_to_next_step() {
        let mut game = make_game_with_petty_cash(100_000, 0);
        // INDUCEMENTS factory default is `true`, so disable it explicitly to exercise the skip path.
        game.options.set(INDUCEMENTS, "false");
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

    fn answer(step: &mut StepBuyInducements, game: &mut Game, home: bool,
              purchases: Vec<InducementPurchase>) -> StepOutcome {
        step.handle_command(&Action::BuyInducements { home, purchases }, game, &mut GameRng::new(0))
    }

    /// Java `init`: the team WITHOUT petty cash (the overdog) is asked first, from its treasury
    /// (overdog spending is allowed by default, `useUnlimitedTreasury = true`).
    #[test]
    fn overdog_is_asked_first_from_its_treasury() {
        let mut game = make_game_with_petty_cash(100_000, 0);
        game.team_away.treasury = 30_000;
        let mut step = StepBuyInducements::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(step.phase_name, "AWAY");
        match out.prompt {
            Some(AgentPrompt::BuyPrayersAndInducements { team_id, budget, .. }) => {
                assert_eq!(team_id, "away");
                assert_eq!(budget, 30_000);
            }
            other => panic!("expected BuyPrayersAndInducements prompt, got {other:?}"),
        }
    }

    /// Java `minimumInducementCost` is 0 (the INDUCEMENT_DUMMY-priced types), so even an overdog
    /// with no treasury at all is shown its dialog — Java reports a purchase for both teams in
    /// every unequal-TV parity game.
    #[test]
    fn overdog_with_no_gold_is_still_shown_its_dialog() {
        let mut game = make_game_with_petty_cash(5_000, 0);
        let mut step = StepBuyInducements::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        match out.prompt {
            Some(AgentPrompt::BuyPrayersAndInducements { team_id, budget, .. }) => {
                assert_eq!(team_id, "away");
                assert_eq!(budget, 0);
            }
            other => panic!("expected the overdog's dialog, got {other:?}"),
        }
    }

    #[test]
    fn prompt_lists_prayers_separately_from_available() {
        use ffb_model::option::game_option_id::INDUCEMENTS;
        let mut game = make_game_with_petty_cash(200_000, 0);
        game.options.set(INDUCEMENTS, "true");
        // Prayers are buyable only with INDUCEMENT_PRAYERS_MAX > 0 (factory default 0).
        game.options.set(ffb_model::option::game_option_id::INDUCEMENT_PRAYERS_MAX, "3");
        let mut step = StepBuyInducements::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        match out.prompt {
            Some(AgentPrompt::BuyPrayersAndInducements { available, prayers, .. }) => {
                assert!(prayers.iter().any(|(id, _)| id == "prayers"));
                assert!(!available.iter().any(|(id, _)| id == "prayers"));
                // Java costs come from the options: bribes INDUCEMENT_BRIBES_COST (100k).
                assert!(available.iter().any(|(id, cost)| id == "bribes" && *cost == 100_000));
                // INDUCEMENT_DUMMY-limited types (availability 0) are not offered.
                assert!(!available.iter().any(|(id, _)| id == "throwARock"));
            }
            other => panic!("expected BuyPrayersAndInducements prompt, got {other:?}"),
        }
    }

    /// Java `getAvailableGold` for the underdog: max(overdogSpend - freeCash, 0) + its petty cash
    /// from the TV difference + freeCash, plus at most MAX_UNDERDOG_ALLOWANCE of its treasury.
    #[test]
    fn underdog_is_asked_second_with_petty_cash_plus_capped_treasury() {
        let mut game = make_game_with_petty_cash(100_000, 0);
        game.team_home.treasury = 80_000;
        let mut step = StepBuyInducements::new();
        step.start(&mut game, &mut GameRng::new(0));
        let out = answer(&mut step, &mut game, false, vec![]);
        assert_eq!(step.phase_name, "HOME");
        match out.prompt {
            Some(AgentPrompt::BuyPrayersAndInducements { team_id, budget, .. }) => {
                assert_eq!(team_id, "home");
                assert_eq!(budget, 100_000 + MAX_UNDERDOG_ALLOWANCE);
            }
            other => panic!("expected BuyPrayersAndInducements prompt for home, got {other:?}"),
        }
    }

    /// Java `handleCommand` (not parallel): each answer is reported immediately, so the overdog's
    /// report precedes the underdog's, and Bribery and Corruption comes after both (leaveStep).
    #[test]
    fn each_answer_is_reported_in_answer_order() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game_with_petty_cash(5_000, 0);
        game.team_home.team_value = 1_095_000;
        game.team_away.team_value = 1_100_000;
        game.team_home.special_rules.push("Bribery and Corruption".into());
        let mut step = StepBuyInducements::new();
        step.start(&mut game, &mut GameRng::new(0));
        answer(&mut step, &mut game, false, vec![]);
        let out = answer(&mut step, &mut game, true, vec![]);
        assert_eq!(out.action, StepAction::NextStep);
        let reports = game.report_list.get_reports();
        let ids: Vec<ReportId> = reports.iter().map(|r| r.get_id()).collect();
        assert_eq!(ids, vec![
            ReportId::PRAYERS_AND_INDUCEMENTS_BOUGHT,
            ReportId::PRAYERS_AND_INDUCEMENTS_BOUGHT,
            ReportId::BRIBERY_AND_CORRUPTION_RE_ROLL,
        ]);
        let first = reports[0].to_json().unwrap();
        assert_eq!(first["teamId"], "away");
        assert_eq!(first["gold"], 0);
        assert_eq!(first["teamValue"], 1_100_000);
        let second = reports[1].to_json().unwrap();
        assert_eq!(second["teamId"], "home");
        assert_eq!(second["teamValue"], 1_095_000);
    }

    #[test]
    fn both_sides_submitting_advances_to_next_step() {
        let mut game = make_game_with_petty_cash(200_000, 0);
        let mut step = StepBuyInducements::new();
        step.start(&mut game, &mut GameRng::new(0));
        answer(&mut step, &mut game, false, vec![]);
        let out = answer(&mut step, &mut game, true,
            vec![InducementPurchase { id: "bribes".into(), count: 1 }]);
        assert_eq!(out.action, StepAction::NextStep);
        // Java `turnData.getInducementSet().add(set)`: the bribe lives in the turn's InducementSet.
        assert_eq!(game.turn_data_home.inducement_set.value(Usage::AVOID_BAN), 1);
    }

    /// Java `handleTeamInducements` charges `value × getActualCostId(team)` and never touches the
    /// team's treasury (leaveStep records `treasurySpentOnInducements` instead).
    #[test]
    fn purchase_tracks_used_gold_and_leaves_the_treasury_alone() {
        let mut game = make_game_with_petty_cash(200_000, 0);
        game.team_home.treasury = 500_000;
        let mut step = StepBuyInducements::new();
        step.start(&mut game, &mut GameRng::new(0));
        answer(&mut step, &mut game, false, vec![]);
        answer(&mut step, &mut game, true, vec![InducementPurchase { id: "bribes".into(), count: 1 }]);
        assert_eq!(game.team_home.treasury, 500_000);
        assert_eq!(step.used_inducement_gold_home, 100_000);
    }

    #[test]
    fn prayers_purchase_tracked_separately_from_inducement_set() {
        let mut game = make_game_with_petty_cash(200_000, 0);
        let mut step = StepBuyInducements::new();
        step.start(&mut game, &mut GameRng::new(0));
        answer(&mut step, &mut game, false, vec![]);
        answer(&mut step, &mut game, true, vec![InducementPurchase { id: "prayers".into(), count: 2 }]);
        assert_eq!(step.prayers_bought_home, 2);
        assert!(game.turn_data_home.inducement_set.get("prayers").is_none());
        // 2 × INDUCEMENT_PRAYERS_COST (factory default 50k): charged before the prayers leave the set.
        assert_eq!(step.used_inducement_gold_home, 100_000);
    }

    /// Java takes the command's prayer count as sent (`prayersBought = prayers.getValue()`); the
    /// client enforces the maximum.
    #[test]
    fn prayers_purchase_value_is_taken_as_sent() {
        let mut game = make_game_with_petty_cash(1_000_000, 0);
        let mut step = StepBuyInducements::new();
        step.start(&mut game, &mut GameRng::new(0));
        answer(&mut step, &mut game, false, vec![]);
        answer(&mut step, &mut game, true, vec![InducementPurchase { id: "prayers".into(), count: 4 }]);
        assert_eq!(step.prayers_bought_home, 4);
    }

    #[test]
    fn leave_step_publishes_prayers_bought_parameters() {
        let mut game = make_game_with_petty_cash(200_000, 0);
        let mut step = StepBuyInducements::new();
        step.start(&mut game, &mut GameRng::new(0));
        answer(&mut step, &mut game, false, vec![]);
        let out = answer(&mut step, &mut game, true,
            vec![InducementPurchase { id: "prayers".into(), count: 1 }]);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::PrayersBoughtHome(1))));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::PrayersBoughtAway(0))));
    }

    /// The pushed kickoff is the certified game-opening kickoff (coin toss first).
    #[test]
    fn leave_step_pushes_the_opening_kickoff_with_the_coin_toss() {
        let mut game = make_game_with_petty_cash(5_000, 0);
        let mut step = StepBuyInducements::new();
        step.start(&mut game, &mut GameRng::new(0));
        answer(&mut step, &mut game, false, vec![]);
        let out = answer(&mut step, &mut game, true, vec![]);
        let kickoff = &out.pushes[0];
        assert_eq!(kickoff[0].step_id, StepId::CoinChoice);
        assert_eq!(kickoff.len(), crate::step::sequences::opening_kickoff_sequence_for(Rules::Bb2025).len());
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
