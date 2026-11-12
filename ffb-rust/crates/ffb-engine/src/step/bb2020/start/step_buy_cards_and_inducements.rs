/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2020.start.StepBuyCardsAndInducements`.
///
/// Step in start game sequence to buy cards and inducements (BB2020).
///
/// Sets stepParameter INDUCEMENT_GOLD_AWAY / INDUCEMENT_GOLD_HOME for all steps on the stack.
///
/// Java phase machine:
///   INIT → determine who has petty cash (or both via allowEvenCTV), set phase HOME or AWAY.
///   HOME / AWAY → show card/inducement dialog; handle CLIENT_SELECT_CARD_TO_BUY /
///                 CLIENT_BUY_INDUCEMENTS; swap team on completion.
///   DONE → leaveStep() — push Kickoff + Inducement(AFTER_INDUCEMENTS_PURCHASED) × 2 +
///           RiotousRookies sequences; record treasury/pettyCash spent; NEXT_STEP.
///
/// Inducement purchases are data-driven from `data/inducements/bb2020_inducements.json` (fixed
/// costs; see `util_inducement_catalog`) rather than a ported `InducementTypeFactory`. Dialogs
/// are shown sequentially (whichever team's `phase` is active) via `AgentPrompt::BuyInducements`.
///
/// In "parallel" mode (equal-TV games with allowEvenCTV option set) both teams' budgets are
/// computed up front, but — since the driver only supports one pending prompt at a time — this
/// translation still resolves the dialogs sequentially rather than truly simultaneously; the
/// `buy_inducement_commands` buffer is unused (a known, narrower-path simplification).
///
/// USE_PREDEFINED_INDUCEMENTS: skips dialog (predefined set application requires InducementTypeFactory — not ported).
/// CardTypeFactory / CardDeck / card-choice randomisation — card system not ported; card buying
/// via `CLIENT_SELECT_CARD_TO_BUY` is out of scope for this pass (see `handle_card`).
/// headless: BuyInducements action lacks star_player_position_ids / mercenary_position_ids —
///   add_star_players / add_mercenaries / add_staff are implemented but require extended action
///   fields not yet present in the Rust action model (star players / infamous staff purchasing
///   is out of scope for this pass — `roster_star`/`roster_staff` catalog entries are filtered
///   out of the buyable list entirely, see `util_inducement_catalog`).
/// BriberyAndCorruption: adds 1 briberyAndCorruption inducement if team has the special rule.
/// rerollOnesOnKOs: adds 1 bugmansXXXXXX inducement if any team player has canReRollOnesOnKORecovery.
use std::collections::HashMap;
use ffb_model::data::loader::{find_position, BB2020_INDUCEMENTS};
use ffb_model::enums::{InducementPhase, PlayerType, PlayerState, PS_RESERVE};
use ffb_model::events::GameEvent;
use ffb_model::inducement::usage::Usage;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::skill_def::{SkillId, SkillWithValue};
use ffb_model::model::turn_data::TurnData;
use ffb_model::prompts::AgentPrompt;
use ffb_model::util::util_box::UtilBox;
use ffb_model::report::bb2020::report_cards_and_inducements_bought::ReportCardsAndInducementsBought;
use ffb_model::report::mixed::report_bribery_and_corruption_re_roll::ReportBriberyAndCorruptionReRoll;
use ffb_model::option::game_option_id::{
    INDUCEMENTS, FREE_INDUCEMENT_CASH, FREE_CARD_CASH,
    INDUCEMENTS_ALLOW_SPENDING_TREASURY_ON_EQUAL_CTV, INDUCEMENTS_ALLOW_OVERDOG_SPENDING,
    USE_PREDEFINED_INDUCEMENTS, INDUCEMENT_PRAYERS_AVAILABLE_FOR_UNDERDOG,
    ALLOW_STAR_ON_BOTH_TEAMS, ALLOW_STAFF_ON_BOTH_TEAMS,
    INDUCEMENT_MERCENARIES_EXTRA_COST, INDUCEMENT_MERCENARIES_SKILL_COST,
    INDUCEMENTS_ALWAYS_USE_TREASURY,
};
use ffb_model::option::util_game_option::{is_option_enabled, get_int_option};
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::generator::sequence::SequenceStep;
use crate::step::generator::common::inducement::InducementParams;
use crate::step::generator::common::{Inducement, RiotousRookies};
use crate::step::game::start::util_inducement_catalog::{apply_purchases, available_list};
use crate::step::game::start::util_minimum_inducement_cost::minimum_inducement_cost;

/// Phase enum mirroring the private inner `Phase` in Java.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// Java: INIT
    Init,
    /// Java: HOME — home team is buying.
    Home,
    /// Java: AWAY — away team is buying.
    Away,
    /// Java: DONE — both done; ready to leave step.
    Done,
}

impl Phase {
    fn from_name(s: &str) -> Self {
        match s {
            "HOME"  => Phase::Home,
            "AWAY"  => Phase::Away,
            "DONE"  => Phase::Done,
            _       => Phase::Init,
        }
    }

    fn as_name(self) -> &'static str {
        match self {
            Phase::Init => "INIT",
            Phase::Home => "HOME",
            Phase::Away => "AWAY",
            Phase::Done => "DONE",
        }
    }
}

/// Java: `StepBuyCardsAndInducements` (bb2020/start).
pub struct StepBuyCardsAndInducements {
    /// Java: `availableInducementGoldHome` — None until set.
    pub available_inducement_gold_home: Option<i32>,
    /// Java: `availableInducementGoldAway` — None until set.
    pub available_inducement_gold_away: Option<i32>,
    /// Java: `usedInducementGoldHome` (init 0).
    pub used_inducement_gold_home: i32,
    /// Java: `usedInducementGoldAway` (init 0).
    pub used_inducement_gold_away: i32,
    /// Java: `parallel` — true when both teams buy simultaneously (equal CTV).
    pub parallel: bool,
    /// Java: `phase` — current buying phase.
    pub phase: Phase,
    /// Java: `buyInducementCommands` — buffered in parallel mode.
    pub buy_inducement_commands: Vec<String>,
    /// Java: `currentSelection` — the card the coach just chose (None = not choosing card).
    pub current_selection: Option<String>,
    /// The dialog `showDialog` opened and that is still waiting for its answer:
    /// (home?, availableGold). Java hands it to `UtilServerDialog.showDialog`; here it becomes
    /// the step's prompt.
    dialog: Option<(bool, i32)>,
}

impl StepBuyCardsAndInducements {
    pub fn new() -> Self {
        Self {
            available_inducement_gold_home: None,
            available_inducement_gold_away: None,
            used_inducement_gold_home: 0,
            used_inducement_gold_away: 0,
            parallel: false,
            phase: Phase::Init,
            buy_inducement_commands: Vec::new(),
            current_selection: None,
            dialog: None,
        }
    }

    /// Java: `executeStep()` — main state machine dispatch.
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.dialog = None;
        match self.phase {
            Phase::Init => self.init(game),
            Phase::Home | Phase::Away => {
                if self.current_selection.is_some() {
                    self.handle_card()
                } else {
                    self.swap_team(game)
                }
            }
            Phase::Done => {}
        }

        if self.phase == Phase::Done {
            self.leave_step(game, rng)
        } else {
            self.build_prompt(game)
        }
    }

    /// Java: `getAvailableGold(int freeCash, boolean useTreasury, boolean allowSpending)` for the
    /// team whose phase is active: its treasury (plus free cash), or the overdog's spend plus its
    /// own petty cash from the team-value difference.
    fn get_available_gold(&mut self, game: &Game, free_cash: i32, use_treasury: bool, allow_spending: bool) -> i32 {
        let home = self.phase == Phase::Home;
        let available = if !allow_spending {
            0
        } else if home {
            if use_treasury { free_cash + game.team_home.treasury }
            else { self.used_inducement_gold_away + game.game_result.home.petty_cash_from_tv_diff }
        } else if use_treasury {
            free_cash + game.team_away.treasury
        } else {
            self.used_inducement_gold_home + game.game_result.away.petty_cash_from_tv_diff
        };
        if home {
            self.available_inducement_gold_home = Some(available);
        } else {
            self.available_inducement_gold_away = Some(available);
        }
        available
    }

    /// Java: `showDialog(Team, freeCash, usesTreasury, allowSpending)` — opens the team's dialog
    /// when it can buy a card or afford the cheapest inducement.
    ///
    /// `canBuyCards` needs `MAX_NR_OF_CARDS > 0`, gold of at least CARDS_SPECIAL_PLAY_COST (100k)
    /// and a deck with more than one card. Java then draws card choices with six `rollDice` calls
    /// (`updateChoices`); the card system is not ported (see module docs), so this translation
    /// cannot take that branch. No drafted squad reaches it: the largest treasury is 50k and no
    /// team-value gap reaches 100k (checked over every cross game of 2026-11-12's sweep).
    fn show_dialog(&mut self, game: &Game, home: bool, free_cash: i32, uses_treasury: bool, allow_spending: bool) -> bool {
        let available_gold = self.get_available_gold(game, free_cash, uses_treasury, allow_spending);
        let team = if home { &game.team_home } else { &game.team_away };
        let can_buy_inducements = minimum_inducement_cost(game, team) <= available_gold;
        if can_buy_inducements {
            self.dialog = Some((home, available_gold));
            return true;
        }
        false
    }

    /// The open `DialogBuyCardsAndInducementsParameter` as a prompt. Card drawing/choice is out of
    /// scope (see module docs), so `available` here only carries the catalog-driven inducements.
    fn build_prompt(&self, game: &Game) -> StepOutcome {
        let (home, budget) = match self.dialog {
            Some(d) => d,
            // Java never leaves executeStep in HOME/AWAY without an open dialog.
            None => return StepOutcome::cont(),
        };
        let team = if home { &game.team_home } else { &game.team_away };
        let available = available_list(&BB2020_INDUCEMENTS.inducements, team);
        StepOutcome::cont().with_prompt(AgentPrompt::BuyInducements {
            team_id: team.id.clone(),
            available,
            budget,
        })
    }

    /// Java: `handleBuyInducements(Game, ClientCommandBuyInducements)` — applies the purchase
    /// list against the catalog (star players / mercenaries / infamous staff / cards are
    /// handled via separate action fields not modeled here — see module docs). Returns a
    /// `GameEvent::BuyInducement` per inducement id actually purchased.
    fn apply_purchase(&mut self, game: &mut Game, home: bool, purchases: &[crate::action::InducementPurchase]) -> Vec<GameEvent> {
        let catalog = &BB2020_INDUCEMENTS.inducements;
        let (team_id, spent, applied) = if home {
            let budget = self.available_inducement_gold_home.unwrap_or(0) - self.used_inducement_gold_home;
            let (spent, applied) = apply_purchases(catalog, &mut game.team_home, &mut game.turn_data_home.inducement_set, purchases, budget);
            (game.team_home.id.clone(), spent, applied)
        } else {
            let budget = self.available_inducement_gold_away.unwrap_or(0) - self.used_inducement_gold_away;
            let (spent, applied) = apply_purchases(catalog, &mut game.team_away, &mut game.turn_data_away.inducement_set, purchases, budget);
            (game.team_away.id.clone(), spent, applied)
        };
        if home { self.used_inducement_gold_home += spent; } else { self.used_inducement_gold_away += spent; }
        applied.into_iter()
            .map(|(id, count)| GameEvent::BuyInducement { team_id: team_id.clone(), inducement_id: id, count })
            .collect()
    }

    /// Java: `init(Game game)` — determine who has petty cash and set initial phase.
    fn init(&mut self, game: &mut Game) {
        // Java: if (!INDUCEMENTS) → phase = DONE
        if !is_option_enabled(game, INDUCEMENTS) {
            self.phase = Phase::Done;
            self.available_inducement_gold_home = Some(0);
            self.available_inducement_gold_away = Some(0);
            return;
        }

        // Java: if (USE_PREDEFINED_INDUCEMENTS) → apply predefined sets, skip dialog
        // no-op: InducementTypeFactory not ported — headless auto-skips inducement dialog
        if is_option_enabled(game, USE_PREDEFINED_INDUCEMENTS) {
            self.phase = Phase::Done;
            self.available_inducement_gold_home = Some(0);
            self.available_inducement_gold_away = Some(0);
            return;
        }
        // no-op: buildDecks() — CardTypeFactory/CardDeck not ported

        let free_cash = get_int_option(game, FREE_INDUCEMENT_CASH) + get_int_option(game, FREE_CARD_CASH);

        let petty_home = game.game_result.home.petty_cash_from_tv_diff;
        let petty_away = game.game_result.away.petty_cash_from_tv_diff;

        let allow_even_ctv = is_option_enabled(game, INDUCEMENTS_ALLOW_SPENDING_TREASURY_ON_EQUAL_CTV)
            || free_cash > 0;
        let mut same_tv = false;

        // Java: the team WITHOUT petty cash is the overdog and is asked first.
        let overdog_home = if petty_home > 0 {
            self.phase = Phase::Away;
            false
        } else if petty_away > 0 {
            self.phase = Phase::Home;
            true
        } else if allow_even_ctv {
            self.phase = Phase::Home;
            self.parallel = true;
            same_tv = true;
            true
        } else {
            // Equal TV and no treasury spending: skip.
            self.available_inducement_gold_home = Some(0);
            self.available_inducement_gold_away = Some(0);
            self.phase = Phase::Done;
            return;
        };

        let allow_overdog = is_option_enabled(game, INDUCEMENTS_ALLOW_OVERDOG_SPENDING);
        if !self.show_dialog(game, overdog_home, free_cash, true, allow_overdog || same_tv) {
            self.swap_team(game);
        }
    }

    /// Java: `handleCard()` — apply the selected card and refresh choices.
    fn handle_card(&mut self) {
        // no-op: full card handling (CardDeck) not ported
        // Java: deduct cardPrice from gold, add chosen card to inducement set,
        //       call updateChoices() to draw next pair for the coach.
        self.current_selection = None;
    }

    /// Java: `swapTeam()` — move to the next team (computing its budget on first visit) or DONE.
    fn swap_team(&mut self, game: &Game) {
        let home = match self.phase {
            Phase::Home if self.available_inducement_gold_away.is_none() => {
                self.phase = Phase::Away;
                false
            }
            Phase::Away if self.available_inducement_gold_home.is_none() => {
                self.phase = Phase::Home;
                true
            }
            _ => {
                self.phase = Phase::Done;
                return;
            }
        };
        let free_cash = get_int_option(game, FREE_INDUCEMENT_CASH) + get_int_option(game, FREE_CARD_CASH);
        let uses_treasury = self.parallel || is_option_enabled(game, INDUCEMENTS_ALWAYS_USE_TREASURY);
        if !self.show_dialog(game, home, free_cash, uses_treasury, true) {
            self.phase = Phase::Done;
        }
    }

    /// Java: `generateReport(Team, gold, newTv)`.
    fn generate_report(game: &Game, home: bool, gold: i32, new_tv: i32) -> ReportCardsAndInducementsBought {
        let (team_id, inducement_set) = if home {
            (game.team_home.id.clone(), &game.turn_data_home.inducement_set)
        } else {
            (game.team_away.id.clone(), &game.turn_data_away.inducement_set)
        };
        let mut nr_of_inducements = 0;
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
        // `inducementSet.getAllCards().length`: no card is ever bought (card system not ported).
        ReportCardsAndInducementsBought::new(team_id, 0, nr_of_inducements, nr_of_stars, nr_of_mercenaries, gold, new_tv)
    }

    /// Java: `leaveStep()` — push sequences, record gold spent, NEXT_STEP.
    fn leave_step(&self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // no-op: apply buffered buyInducementCommands — `buy_inducement_commands` buffering is
        // unused in this translation; purchases are applied immediately in `handle_command` /
        // `apply_purchase` instead (see module docs on parallel-mode simplification).

        let new_tv_home = game.team_home.team_value
            + self.used_inducement_gold_home;
        let new_tv_away = game.team_away.team_value
            + self.used_inducement_gold_away;

        // Java: if parallel → addReport for both teams now (serial: already reported per-command).
        if self.parallel {
            let report = Self::generate_report(game, true, self.used_inducement_gold_home, new_tv_home);
            game.report_list.add(report);
            let report = Self::generate_report(game, false, self.used_inducement_gold_away, new_tv_away);
            game.report_list.add(report);
        }

        // Java: ((Kickoff) factory.forName(Kickoff)).pushSequence(new Kickoff.SequenceParams(gameState, true))
        // — the game-opening kickoff with the coin toss, in the certified form (see
        // `sequences::opening_kickoff_sequence_for`).
        let seq_kickoff = crate::step::sequences::opening_kickoff_sequence_for(game.rules);

        // Java: push Inducement(AFTER_INDUCEMENTS_PURCHASED) × 2.
        // Order: if newTvHome > newTvAway → home first; else away first.
        let (first_home, second_home) = if new_tv_home > new_tv_away {
            (true, false)
        } else {
            (false, true)
        };
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

        // Java: push RiotousRookies sequence.
        let seq_riotous = RiotousRookies::build_sequence();

        // Java: check INDUCEMENT_PRAYERS_AVAILABLE_FOR_UNDERDOG option → push PRAYERS step.
        let use_prayers = is_option_enabled(game, INDUCEMENT_PRAYERS_AVAILABLE_FOR_UNDERDOG);
        let seq_prayers = if use_prayers {
            Some(vec![SequenceStep::with_params(StepId::Prayers, vec![
                StepParameter::TvHome(new_tv_home),
                StepParameter::TvAway(new_tv_away),
            ])])
        } else {
            None
        };

        // Java: a team without petty cash (or when the treasury is always used) spent treasury;
        // the underdog's spend is all petty cash (`setPettyCashUsed(usedInducementGold)`).
        let always_use_treasury = is_option_enabled(game, INDUCEMENTS_ALWAYS_USE_TREASURY);
        let free_cash = get_int_option(game, FREE_INDUCEMENT_CASH) + get_int_option(game, FREE_CARD_CASH);
        if game.game_result.home.petty_cash_from_tv_diff == 0 || always_use_treasury {
            game.game_result.home.treasury_spent_on_inducements = (self.used_inducement_gold_home - free_cash).max(0);
        } else {
            game.game_result.home.petty_cash_used = self.used_inducement_gold_home;
        }
        if game.game_result.away.petty_cash_from_tv_diff == 0 || always_use_treasury {
            game.game_result.away.treasury_spent_on_inducements = (self.used_inducement_gold_away - free_cash).max(0);
        } else {
            game.game_result.away.petty_cash_used = self.used_inducement_gold_away;
        }

        // Java: inducementTypeFactory.allTypes() filtered by Usage.REROLL_ARGUE → "briberyAndCorruption".
        // If team has BriberyAndCorruption special rule → add 1 briberyAndCorruption inducement.
        {
            use ffb_model::inducement::inducement::Inducement as InducementModel;
            use ffb_model::inducement::usage::Usage;
            use ffb_model::model::special_rule::SpecialRule;
            let bnc_name = SpecialRule::BRIBERY_AND_CORRUPTION.get_rule_name();
            if game.team_home.special_rules.iter().any(|r| r == bnc_name) {
                game.turn_data_home.inducement_set.add_inducement(
                    InducementModel::new("briberyAndCorruption", 1, vec![Usage::REROLL_ARGUE]));
                // Java: getResult().addReport(new ReportBriberyAndCorruptionReRoll(teamHome.getId(), BriberyAndCorruptionAction.ADDED))
                game.report_list.add(ReportBriberyAndCorruptionReRoll::new(
                    Some(game.team_home.id.clone()),
                    "ADDED".into(),
                ));
            }
            if game.team_away.special_rules.iter().any(|r| r == bnc_name) {
                game.turn_data_away.inducement_set.add_inducement(
                    InducementModel::new("briberyAndCorruption", 1, vec![Usage::REROLL_ARGUE]));
                // Java: getResult().addReport(new ReportBriberyAndCorruptionReRoll(teamAway.getId(), BriberyAndCorruptionAction.ADDED))
                game.report_list.add(ReportBriberyAndCorruptionReRoll::new(
                    Some(game.team_away.id.clone()),
                    "ADDED".into(),
                ));
            }
        }

        // Java: inducementTypeFactory.allTypes() filtered by Usage.REROLL_ONES_ON_KOS → "bugmansXXXXXX".
        // If any home/away player has canReRollOnesOnKORecovery → add 1 bugmansXXXXXX inducement.
        {
            use ffb_model::inducement::inducement::Inducement as InducementModel;
            use ffb_model::inducement::usage::Usage;
            use ffb_model::model::property::NamedProperties;
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

        // Java: publishParameter(TV_HOME / TV_AWAY) only together with the PRAYERS sequence.
        // Java: setNextAction(StepAction.NEXT_STEP)
        let mut out = StepOutcome::next()
            .push_seq(seq_kickoff)
            .push_seq(seq_first)
            .push_seq(seq_second)
            .push_seq(seq_riotous);
        if let Some(seq) = seq_prayers {
            out = out.push_seq(seq)
                .publish(StepParameter::TvHome(new_tv_home))
                .publish(StepParameter::TvAway(new_tv_away));
        }
        out
    }

    /// Java: `addStarPlayers(Team, String[] positionIds)` — creates new Star players from roster
    /// positions and adds them to the team. Returns the cost sum for all added stars.
    /// Removes any duplicate star from the opposing team if ALLOW_STAR_ON_BOTH_TEAMS is disabled.
    ///
    /// engine: GameEvent::PlayerAdded emitted (Phase ZT — requires method to propagate events).
    /// no-op: ReportDoubleHiredStarPlayer — reports deferred.
    fn add_star_players(&self, game: &mut Game, home: bool, position_ids: &[String]) -> i32 {
        let mut sum = 0i32;
        if position_ids.is_empty() { return sum; }

        let team_id = if home { game.team_home.id.clone() } else { game.team_away.id.clone() };
        let roster_id = if home { game.team_home.roster_id.clone() } else { game.team_away.roster_id.clone() };
        let allow_both = is_option_enabled(game, ALLOW_STAR_ON_BOTH_TEAMS);

        let other_star_by_name: HashMap<String, String> = {
            let other = if home { &game.team_away } else { &game.team_home };
            other.players.iter()
                .filter(|p| p.player_type == PlayerType::Star)
                .map(|p| (p.name.clone(), p.id.clone()))
                .collect()
        };

        let mut to_add: Vec<Player> = Vec::new();
        let mut removed_ids: Vec<String> = Vec::new();
        let mut current_max_nr = {
            let team = if home { &game.team_home } else { &game.team_away };
            team.players.iter().map(|p| p.nr).max().unwrap_or(0)
        };

        for position_id in position_ids {
            let position = match find_position(&roster_id, position_id, game.rules) {
                Some(p) => p,
                None => continue,
            };
            sum += position.cost;
            if !allow_both {
                if let Some(other_id) = other_star_by_name.get(&position.name) {
                    removed_ids.push(other_id.clone());
                    continue;
                }
            }
            current_max_nr += 1;
            let idx = to_add.len() + 1;
            let mut player = Player::default();
            player.id = format!("{}S{}", team_id, idx);
            player.name = position.name.clone();
            player.nr = current_max_nr;
            player.position_id = position.id.clone();
            player.player_type = PlayerType::Star;
            player.movement = position.ma;
            player.strength = position.st;
            player.agility = position.ag;
            player.passing = position.pa;
            player.armour = position.av;
            player.position_movement = position.ma;
            player.position_strength = position.st;
            player.position_agility = position.ag;
            player.position_passing = position.pa;
            player.position_armour = position.av;
            player.starting_skills = position.skills.iter()
                .filter_map(|e| SkillId::from_class_name(e.name()).map(|skill_id| SkillWithValue { skill_id, value: None }))
                .collect();
            to_add.push(player);
        }

        let removed_count = removed_ids.len() as i32;
        for id in &removed_ids {
            if home {
                game.team_away.players.retain(|p| &p.id != id);
            } else {
                game.team_home.players.retain(|p| &p.id != id);
            }
            game.field_model.remove_player(id);
            // no-op: server.getCommunication().sendRemovePlayer() — headless engine
            // no-op: getResult().addReport(new ReportDoubleHiredStarPlayer(...)) — headless engine
        }
        if removed_count > 0 {
            Self::remove_duplicate_player_inducements(&mut game.turn_data_home, removed_count, Usage::STAR);
            Self::remove_duplicate_player_inducements(&mut game.turn_data_away, removed_count, Usage::STAR);
        }

        for player in to_add {
            let player_id = player.id.clone();
            if home { game.team_home.players.push(player); } else { game.team_away.players.push(player); }
            game.field_model.set_player_state(&player_id, PlayerState::new(PS_RESERVE));
            UtilBox::put_player_into_box(game, &player_id);
        }
        // engine: emit GameEvent::PlayerAdded per player (Phase ZT — requires method to propagate events)
        sum
    }

    /// Java: `addMercenaries(Team, String[] positionIds, Skill[] skills)` — creates MERCENARY
    /// players with the Loner skill and optional extra skills. Returns the cost sum.
    ///
    /// engine: GameEvent::PlayerAdded emitted (Phase ZT — requires method to propagate events).
    fn add_mercenaries(&self, game: &mut Game, home: bool, position_ids: &[String], extra_skills: &[Option<SkillId>]) -> i32 {
        let mut sum = 0i32;
        if position_ids.is_empty() || extra_skills.is_empty() { return sum; }

        let team_id = if home { game.team_home.id.clone() } else { game.team_away.id.clone() };
        let roster_id = if home { game.team_home.roster_id.clone() } else { game.team_away.roster_id.clone() };
        let extra_cost = get_int_option(game, INDUCEMENT_MERCENARIES_EXTRA_COST);
        let skill_cost = get_int_option(game, INDUCEMENT_MERCENARIES_SKILL_COST);

        let mut to_add: Vec<Player> = Vec::new();
        let mut nr_by_pos: HashMap<String, i32> = HashMap::new();
        let mut current_max_nr = {
            let team = if home { &game.team_home } else { &game.team_away };
            team.players.iter().map(|p| p.nr).max().unwrap_or(0)
        };

        for (i, position_id) in position_ids.iter().enumerate() {
            let position = match find_position(&roster_id, position_id, game.rules) {
                Some(p) => p,
                None => continue,
            };
            sum += position.cost + extra_cost;
            current_max_nr += 1;
            let merc_nr = {
                let entry = nr_by_pos.entry(position.id.clone()).or_insert(0);
                *entry += 1;
                *entry
            };
            let idx = to_add.len() + 1;
            let mut player = Player::default();
            player.id = format!("{}M{}", team_id, idx);
            player.name = format!("Merc {} {}", position.name, merc_nr);
            player.nr = current_max_nr;
            player.position_id = position.id.clone();
            player.player_type = PlayerType::Mercenary;
            player.movement = position.ma;
            player.strength = position.st;
            player.agility = position.ag;
            player.passing = position.pa;
            player.armour = position.av;
            player.position_movement = position.ma;
            player.position_strength = position.st;
            player.position_agility = position.ag;
            player.position_passing = position.pa;
            player.position_armour = position.av;
            player.starting_skills = position.skills.iter()
                .filter_map(|e| SkillId::from_class_name(e.name()).map(|skill_id| SkillWithValue { skill_id, value: None }))
                .collect();
            player.add_skill(SkillId::Loner);
            if let Some(Some(skill)) = extra_skills.get(i) {
                player.add_skill(*skill);
                sum += skill_cost;
            }
            to_add.push(player);
        }

        for player in to_add {
            let player_id = player.id.clone();
            if home { game.team_home.players.push(player); } else { game.team_away.players.push(player); }
            game.field_model.set_player_state(&player_id, PlayerState::new(PS_RESERVE));
            UtilBox::put_player_into_box(game, &player_id);
        }
        // engine: emit GameEvent::PlayerAdded per player (Phase ZT — requires method to propagate events)
        sum
    }

    /// Java: `addStaff(Team, List<String> positionIds)` — creates InfamousStaff players from
    /// roster positions. Removes duplicate staff from the opposing team if ALLOW_STAFF_ON_BOTH_TEAMS
    /// is disabled. Returns the cost sum.
    ///
    /// engine: GameEvent::PlayerAdded emitted (Phase ZT — requires method to propagate events).
    /// no-op: ReportDoubleHiredStaff — reports deferred.
    fn add_staff(&self, game: &mut Game, home: bool, position_ids: Vec<String>) -> i32 {
        let mut sum = 0i32;
        if position_ids.is_empty() { return sum; }

        let team_id = if home { game.team_home.id.clone() } else { game.team_away.id.clone() };
        let roster_id = if home { game.team_home.roster_id.clone() } else { game.team_away.roster_id.clone() };
        let allow_both = is_option_enabled(game, ALLOW_STAFF_ON_BOTH_TEAMS);

        let other_staff_by_name: HashMap<String, String> = {
            let other = if home { &game.team_away } else { &game.team_home };
            other.players.iter()
                .filter(|p| p.player_type == PlayerType::InfamousStaff)
                .map(|p| (p.name.clone(), p.id.clone()))
                .collect()
        };

        let mut to_add: Vec<Player> = Vec::new();
        let mut removed_ids: Vec<String> = Vec::new();
        let mut current_max_nr = {
            let team = if home { &game.team_home } else { &game.team_away };
            team.players.iter().map(|p| p.nr).max().unwrap_or(0)
        };

        for position_id in &position_ids {
            let position = match find_position(&roster_id, position_id, game.rules) {
                Some(p) => p,
                None => continue,
            };
            sum += position.cost;
            if !allow_both {
                if let Some(other_id) = other_staff_by_name.get(&position.name) {
                    removed_ids.push(other_id.clone());
                    continue;
                }
            }
            current_max_nr += 1;
            let idx = to_add.len() + 1;
            let mut player = Player::default();
            player.id = format!("{}I{}", team_id, idx);
            player.name = position.name.clone();
            player.nr = current_max_nr;
            player.position_id = position.id.clone();
            player.player_type = PlayerType::InfamousStaff;
            player.movement = position.ma;
            player.strength = position.st;
            player.agility = position.ag;
            player.passing = position.pa;
            player.armour = position.av;
            player.position_movement = position.ma;
            player.position_strength = position.st;
            player.position_agility = position.ag;
            player.position_passing = position.pa;
            player.position_armour = position.av;
            player.starting_skills = position.skills.iter()
                .filter_map(|e| SkillId::from_class_name(e.name()).map(|skill_id| SkillWithValue { skill_id, value: None }))
                .collect();
            to_add.push(player);
        }

        let removed_count = removed_ids.len() as i32;
        for id in &removed_ids {
            if home {
                game.team_away.players.retain(|p| &p.id != id);
            } else {
                game.team_home.players.retain(|p| &p.id != id);
            }
            game.field_model.remove_player(id);
            // no-op: server.getCommunication().sendRemovePlayer() — headless engine
            // no-op: getResult().addReport(new ReportDoubleHiredStaff(...)) — headless engine
        }
        if removed_count > 0 {
            Self::remove_duplicate_player_inducements(&mut game.turn_data_home, removed_count, Usage::STAFF);
            Self::remove_duplicate_player_inducements(&mut game.turn_data_away, removed_count, Usage::STAFF);
        }

        for player in to_add {
            let player_id = player.id.clone();
            if home { game.team_home.players.push(player); } else { game.team_away.players.push(player); }
            game.field_model.set_player_state(&player_id, PlayerState::new(PS_RESERVE));
            UtilBox::put_player_into_box(game, &player_id);
        }
        // engine: emit GameEvent::PlayerAdded per player (Phase ZT — requires method to propagate events)
        sum
    }

    /// Java: `removeDuplicatePlayerInducements(TurnData, int removed, Usage usage)` — reduces the
    /// inducement count for the first matching usage; removes it if count reaches zero or below.
    fn remove_duplicate_player_inducements(turn_data: &mut TurnData, removed: i32, usage: Usage) {
        if let Some(type_id) = turn_data.inducement_set.for_usage(usage).map(|s| s.to_string()) {
            let current = turn_data.inducement_set.get(&type_id).map_or(0, |i| i.get_value());
            let new_val = current - removed;
            if new_val <= 0 {
                turn_data.inducement_set.remove_inducement(&type_id);
            } else if let Some(mut ind) = turn_data.inducement_set.get(&type_id) {
                ind.value = new_val;
                turn_data.inducement_set.add_inducement(ind);
            }
        }
    }
}

impl Default for StepBuyCardsAndInducements {
    fn default() -> Self { Self::new() }
}

impl Step for StepBuyCardsAndInducements {
    fn id(&self) -> StepId { StepId::BuyCardsAndInducements }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: super.start(); executeStep()
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: commandStatus = super.handleCommand(...)
        // Java: switch (pReceivedCommand.getId()):
        //   CLIENT_SELECT_CARD_TO_BUY → currentSelection = command.getSelection(); EXECUTE_STEP
        //   CLIENT_BUY_INDUCEMENTS →
        //     if parallel: buffer command
        //     else: handleBuyInducements + addReport
        //     EXECUTE_STEP
        let mut events = Vec::new();
        if let Action::BuyInducements { home, purchases } = action {
            events = self.apply_purchase(game, *home, purchases);
            if !self.parallel {
                // Java: `getNewTv(used, turnData, team)` = team value + gold used + card cost
                // (no card is ever bought here), then `addReport(generateReport(team, used, newTv))`.
                let (used, team_value) = if *home {
                    (self.used_inducement_gold_home, game.team_home.team_value)
                } else {
                    (self.used_inducement_gold_away, game.team_away.team_value)
                };
                let report = Self::generate_report(game, *home, used, team_value + used);
                game.report_list.add(report);
            }
        }
        let mut out = self.execute_step(game, rng);
        for ev in events { out = out.with_event(ev); }
        out
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::InducementPurchase;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    fn make_game_with_petty_cash(petty_home: i32, petty_away: i32) -> Game {
        let mut game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020);
        game.game_result.home.petty_cash_from_tv_diff = petty_home;
        game.game_result.away.petty_cash_from_tv_diff = petty_away;
        game
    }

    #[test]
    fn initial_phase_is_init() {
        let step = StepBuyCardsAndInducements::new();
        assert_eq!(step.phase, Phase::Init);
    }

    #[test]
    fn equal_tv_no_petty_cash_advances_to_done_and_next_step() {
        // Both teams equal TV → no inducements → should reach Done and return NextStep.
        let mut game = make_game();
        let mut step = StepBuyCardsAndInducements::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(step.phase, Phase::Done);
    }

    /// Java leaveStep publishes TV_HOME / TV_AWAY only together with the PRAYERS sequence
    /// (INDUCEMENT_PRAYERS_AVAILABLE_FOR_UNDERDOG), and never INDUCEMENT_GOLD_*.
    #[test]
    fn tv_is_published_only_with_the_prayers_sequence() {
        let mut game = make_game();
        let mut step = StepBuyCardsAndInducements::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::TvHome(_))));
        assert!(!out.published.iter().any(|p| matches!(p, StepParameter::InducementGoldHome(_))));

        let mut game = make_game();
        game.options.set(INDUCEMENT_PRAYERS_AVAILABLE_FOR_UNDERDOG, "false");
        let mut step = StepBuyCardsAndInducements::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(!out.published.iter().any(|p| matches!(p, StepParameter::TvHome(_))));
        // Kickoff + 2 × Inducement + RiotousRookies, no Prayers.
        assert_eq!(out.pushes.len(), 4);
    }
    #[test]
    fn equal_tv_pushes_two_inducement_sequences_and_riotous_rookies() {
        let mut game = make_game();
        let mut step = StepBuyCardsAndInducements::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Expect: 1 × Kickoff + 2 × Inducement(AFTER_INDUCEMENTS_PURCHASED) + 1 × RiotousRookies +
        // 1 × Prayers = 5 sequences. The Prayers sequence is pushed because
        // INDUCEMENT_PRAYERS_AVAILABLE_FOR_UNDERDOG has factory default `true`.
        assert_eq!(out.pushes.len(), 5, "expected Kickoff + 2 Inducement + RiotousRookies + Prayers sequences");
    }

    #[test]
    fn phase_is_done_after_equal_tv_start() {
        let mut game = make_game();
        let mut step = StepBuyCardsAndInducements::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(step.phase, Phase::Done);
    }

    fn answer(step: &mut StepBuyCardsAndInducements, game: &mut Game, home: bool,
              purchases: Vec<InducementPurchase>) -> StepOutcome {
        step.handle_command(&Action::BuyInducements { home, purchases }, game, &mut GameRng::new(0))
    }

    /// Java `init`: home has petty cash, so AWAY is the overdog and is asked first, from its
    /// treasury; the underdog's gold is not computed until `swapTeam`.
    #[test]
    fn home_underdog_means_away_overdog_is_asked_first() {
        let mut game = make_game_with_petty_cash(100_000, 0);
        let mut step = StepBuyCardsAndInducements::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(step.phase, Phase::Away);
        assert_eq!(step.available_inducement_gold_away, Some(0));
        assert_eq!(step.available_inducement_gold_home, None);
    }

    #[test]
    fn away_underdog_means_home_overdog_is_asked_first() {
        let mut game = make_game_with_petty_cash(0, 80_000);
        game.team_home.treasury = 40_000;
        let mut step = StepBuyCardsAndInducements::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(step.phase, Phase::Home);
        assert_eq!(step.available_inducement_gold_home, Some(40_000));
    }

    #[test]
    fn inducements_disabled_skips_to_done() {
        let mut game = make_game_with_petty_cash(100_000, 0);
        // INDUCEMENTS factory default is `true`, so disable it explicitly to exercise the skip path.
        game.options.set(INDUCEMENTS, "false");
        let mut step = StepBuyCardsAndInducements::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(step.phase, Phase::Done);
        assert_eq!(step.available_inducement_gold_home, Some(0));
        assert_eq!(step.available_inducement_gold_away, Some(0));
    }

    /// Java `getAvailableGold(useTreasury = false)` for the underdog: the overdog's spend plus
    /// its petty cash from the TV difference.
    #[test]
    fn underdog_gold_is_overdog_spend_plus_petty_cash() {
        let mut game = make_game_with_petty_cash(100_000, 0);
        let mut step = StepBuyCardsAndInducements::new();
        step.start(&mut game, &mut GameRng::new(0));
        answer(&mut step, &mut game, false, vec![]);
        assert_eq!(step.phase, Phase::Home);
        assert_eq!(step.available_inducement_gold_home, Some(100_000));
    }

    /// Free inducement and card cash go to the treasury-funded (overdog) budget only; the
    /// underdog's petty-cash budget does not add them.
    #[test]
    fn free_cash_is_added_to_the_overdog_treasury_budget() {
        use ffb_model::option::game_option_id::{FREE_INDUCEMENT_CASH, FREE_CARD_CASH};
        let mut game = make_game_with_petty_cash(100_000, 0);
        game.options.set(FREE_INDUCEMENT_CASH, "20000");
        game.options.set(FREE_CARD_CASH, "10000");
        let mut step = StepBuyCardsAndInducements::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(step.available_inducement_gold_away, Some(30_000));
        answer(&mut step, &mut game, false, vec![]);
        assert_eq!(step.available_inducement_gold_home, Some(100_000));
    }

    /// Equal TV with treasury spending allowed: parallel mode; home first from its treasury, then
    /// away also from its treasury (`parallel` forces useTreasury).
    #[test]
    fn allow_even_ctv_enables_parallel_mode() {
        use ffb_model::option::game_option_id::INDUCEMENTS_ALLOW_SPENDING_TREASURY_ON_EQUAL_CTV;
        let mut game = make_game();
        game.options.set(INDUCEMENTS_ALLOW_SPENDING_TREASURY_ON_EQUAL_CTV, "true");
        game.team_home.treasury = 50_000;
        game.team_away.treasury = 30_000;
        let mut step = StepBuyCardsAndInducements::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(step.parallel);
        assert_eq!(step.available_inducement_gold_home, Some(50_000));
        answer(&mut step, &mut game, true, vec![]);
        assert_eq!(step.available_inducement_gold_away, Some(30_000));
    }
    #[test]
    fn phase_name_round_trips() {
        for (phase, name) in [
            (Phase::Init, "INIT"),
            (Phase::Home, "HOME"),
            (Phase::Away, "AWAY"),
            (Phase::Done, "DONE"),
        ] {
            assert_eq!(phase.as_name(), name);
            assert_eq!(Phase::from_name(name), phase);
        }
    }

    #[test]
    fn bribery_and_corruption_adds_report_for_home_team() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        // Give home team the Bribery and Corruption special rule.
        game.team_home.special_rules.push("Bribery and Corruption".into());
        let mut step = StepBuyCardsAndInducements::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::BRIBERY_AND_CORRUPTION_RE_ROLL),
            "team with BriberyAndCorruption must add BRIBERY_AND_CORRUPTION_RE_ROLL report");
    }

    #[test]
    fn no_bribery_rule_does_not_add_bnc_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        // No special rules on either team.
        let mut step = StepBuyCardsAndInducements::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!game.report_list.has_report(ReportId::BRIBERY_AND_CORRUPTION_RE_ROLL),
            "team without BriberyAndCorruption must NOT add BRIBERY_AND_CORRUPTION_RE_ROLL report");
    }

    // ── dialog + purchase flow tests ──────────────────────────────────────────

    /// The overdog's dialog is shown even with 0 gold (Java `minimumInducementCost` is 0); the
    /// underdog follows with its petty cash.
    #[test]
    fn overdog_then_underdog_get_buy_inducements_prompts() {
        let mut game = make_game_with_petty_cash(100_000, 0);
        let mut step = StepBuyCardsAndInducements::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        match out.prompt {
            Some(AgentPrompt::BuyInducements { team_id, budget, .. }) => {
                assert_eq!(team_id, "away");
                assert_eq!(budget, 0);
            }
            other => panic!("expected BuyInducements prompt for away, got {other:?}"),
        }
        let out = answer(&mut step, &mut game, false, vec![]);
        match out.prompt {
            Some(AgentPrompt::BuyInducements { team_id, budget, .. }) => {
                assert_eq!(team_id, "home");
                assert_eq!(budget, 100_000);
            }
            other => panic!("expected BuyInducements prompt for home, got {other:?}"),
        }
    }

    /// Java `handleCommand` (not parallel) reports each answer immediately: the overdog's report
    /// first, with `teamValue` = team value + gold used.
    #[test]
    fn each_answer_is_reported_in_answer_order() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game_with_petty_cash(5_000, 0);
        game.team_home.team_value = 1_095_000;
        game.team_away.team_value = 1_100_000;
        let mut step = StepBuyCardsAndInducements::new();
        step.start(&mut game, &mut GameRng::new(0));
        answer(&mut step, &mut game, false, vec![]);
        let out = answer(&mut step, &mut game, true, vec![]);
        assert_eq!(out.action, StepAction::NextStep);
        let reports = game.report_list.get_reports();
        assert_eq!(reports.len(), 2);
        assert!(reports.iter().all(|r| r.get_id() == ReportId::CARDS_AND_INDUCEMENTS_BOUGHT));
        let first = reports[0].to_json().unwrap();
        assert_eq!(first["teamId"], "away");
        assert_eq!(first["teamValue"], 1_100_000);
        assert_eq!(reports[1].to_json().unwrap()["teamId"], "home");
    }

    #[test]
    fn both_sides_submitting_advances_to_next_step() {
        let mut game = make_game_with_petty_cash(200_000, 0);
        let mut step = StepBuyCardsAndInducements::new();
        step.start(&mut game, &mut GameRng::new(0));
        answer(&mut step, &mut game, false, vec![]);
        let out = answer(&mut step, &mut game, true,
            vec![InducementPurchase { id: "bribes".into(), count: 1 }]);
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.team_home.bribes, 1);
    }

    #[test]
    fn purchase_deducts_treasury_and_tracks_used_gold() {
        let mut game = make_game_with_petty_cash(200_000, 0);
        game.team_home.treasury = 500_000;
        let mut step = StepBuyCardsAndInducements::new();
        step.start(&mut game, &mut GameRng::new(0));
        answer(&mut step, &mut game, false, vec![]);
        answer(&mut step, &mut game, true, vec![InducementPurchase { id: "bribes".into(), count: 1 }]);
        assert_eq!(game.team_home.treasury, 400_000);
        assert_eq!(step.used_inducement_gold_home, 100_000);
    }

    #[test]
    fn purchase_routes_weather_mage_into_inducement_set() {
        let mut game = make_game_with_petty_cash(200_000, 0);
        let mut step = StepBuyCardsAndInducements::new();
        step.start(&mut game, &mut GameRng::new(0));
        answer(&mut step, &mut game, false, vec![]);
        answer(&mut step, &mut game, true, vec![InducementPurchase { id: "weatherMage".into(), count: 1 }]);
        assert_eq!(game.turn_data_home.inducement_set.get("weatherMage").map(|i| i.get_value()), Some(1));
    }

    #[test]
    fn purchase_clamps_over_budget_purchase() {
        let mut game = make_game_with_petty_cash(150_000, 0);
        let mut step = StepBuyCardsAndInducements::new();
        step.start(&mut game, &mut GameRng::new(0));
        answer(&mut step, &mut game, false, vec![]);
        // 300,000 requested; only 1 bribe (100,000) fits in a 150,000 budget.
        answer(&mut step, &mut game, true, vec![InducementPurchase { id: "bribes".into(), count: 3 }]);
        assert_eq!(game.team_home.bribes, 1);
        assert_eq!(step.used_inducement_gold_home, 100_000);
    }
    #[test]
    fn leave_step_pushes_kickoff_and_inducement_sequences_after_both_submit() {
        use ffb_model::option::game_option_id::INDUCEMENTS;
        let mut game = make_game_with_petty_cash(200_000, 0);
        game.options.set(INDUCEMENTS, "true");
        let mut step = StepBuyCardsAndInducements::new();
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
        // Kickoff + 2×Inducement + RiotousRookies + Prayers = 5 sequences (Prayers pushed because
        // INDUCEMENT_PRAYERS_AVAILABLE_FOR_UNDERDOG has factory default `true`).
        assert_eq!(out.pushes.len(), 5);
    }

    // ── add_star_players tests ─────────────────────────────────────────────────

    fn bb2020_position_ids_for(roster_id: &str) -> Vec<String> {
        use ffb_model::data::loader::bb2020_rosters;
        bb2020_rosters()
            .into_iter()
            .find(|r| r.id == roster_id)
            .map(|r| r.positions.into_iter()
                .filter(|p| p.player_type == "Regular" || p.player_type == "Irregular")
                .map(|p| p.id)
                .collect())
            .unwrap_or_default()
    }

    fn make_game_bb2020(roster_id: &str) -> Game {
        use ffb_model::data::loader::bb2020_rosters;
        let roster = bb2020_rosters().into_iter().find(|r| r.id == roster_id).unwrap();
        let mut home = test_team("home", 0);
        home.roster_id = roster.id.clone();
        let mut away = test_team("away", 0);
        away.roster_id = roster.id.clone();
        Game::new(home, away, Rules::Bb2020)
    }

    #[test]
    fn add_star_players_empty_returns_zero() {
        let step = StepBuyCardsAndInducements::new();
        let mut game = make_game();
        let cost = step.add_star_players(&mut game, true, &[]);
        assert_eq!(cost, 0);
        assert_eq!(game.team_home.players.len(), 0);
    }

    #[test]
    fn add_star_players_unknown_position_skipped() {
        let step = StepBuyCardsAndInducements::new();
        let mut game = make_game_bb2020("human.lrb6");
        let cost = step.add_star_players(&mut game, true, &["NONEXISTENT".to_string()]);
        assert_eq!(cost, 0);
        assert_eq!(game.team_home.players.len(), 0);
    }

    #[test]
    fn add_star_players_adds_star_player_type() {
        let step = StepBuyCardsAndInducements::new();
        let roster_id = "human.lrb6";
        let pos_ids = bb2020_position_ids_for(roster_id);
        let mut game = make_game_bb2020(roster_id);
        let id = pos_ids[0].clone();
        step.add_star_players(&mut game, true, &[id]);
        assert_eq!(game.team_home.players.len(), 1);
        assert_eq!(game.team_home.players[0].player_type, PlayerType::Star);
    }

    #[test]
    fn add_star_players_id_uses_team_prefix_s() {
        let step = StepBuyCardsAndInducements::new();
        let roster_id = "human.lrb6";
        let pos_ids = bb2020_position_ids_for(roster_id);
        let mut game = make_game_bb2020(roster_id);
        step.add_star_players(&mut game, true, &[pos_ids[0].clone()]);
        assert!(game.team_home.players[0].id.starts_with("home"),
            "player id must start with team id 'home'");
        assert!(game.team_home.players[0].id.contains('S'),
            "player id must contain 'S' for Star");
    }

    #[test]
    fn add_star_players_returns_position_cost() {
        use ffb_model::data::loader::bb2020_rosters;
        let step = StepBuyCardsAndInducements::new();
        let roster_id = "human.lrb6";
        let pos_ids = bb2020_position_ids_for(roster_id);
        let mut game = make_game_bb2020(roster_id);
        let pos_cost = bb2020_rosters().into_iter()
            .find(|r| r.id == roster_id).unwrap()
            .positions.into_iter()
            .find(|p| p.id == pos_ids[0]).unwrap()
            .cost;
        let cost = step.add_star_players(&mut game, true, &[pos_ids[0].clone()]);
        assert_eq!(cost, pos_cost);
    }

    #[test]
    fn add_star_players_copies_stats_from_position() {
        use ffb_model::data::loader::bb2020_rosters;
        let step = StepBuyCardsAndInducements::new();
        let roster_id = "human.lrb6";
        let pos_ids = bb2020_position_ids_for(roster_id);
        let mut game = make_game_bb2020(roster_id);
        let pos = bb2020_rosters().into_iter()
            .find(|r| r.id == roster_id).unwrap()
            .positions.into_iter()
            .find(|p| p.id == pos_ids[0]).unwrap();
        step.add_star_players(&mut game, true, &[pos_ids[0].clone()]);
        let p = &game.team_home.players[0];
        assert_eq!(p.movement, pos.ma);
        assert_eq!(p.strength, pos.st);
        assert_eq!(p.agility, pos.ag);
    }

    #[test]
    fn add_star_players_placed_in_reserve() {
        let step = StepBuyCardsAndInducements::new();
        let roster_id = "human.lrb6";
        let pos_ids = bb2020_position_ids_for(roster_id);
        let mut game = make_game_bb2020(roster_id);
        step.add_star_players(&mut game, true, &[pos_ids[0].clone()]);
        let player_id = &game.team_home.players[0].id;
        let state = game.field_model.player_state(player_id);
        assert!(state.is_some(), "player must be placed in field_model");
    }

    // ── add_mercenaries tests ──────────────────────────────────────────────────

    #[test]
    fn add_mercenaries_empty_returns_zero() {
        let step = StepBuyCardsAndInducements::new();
        let mut game = make_game();
        let cost = step.add_mercenaries(&mut game, true, &[], &[]);
        assert_eq!(cost, 0);
        assert_eq!(game.team_home.players.len(), 0);
    }

    #[test]
    fn add_mercenaries_adds_loner_skill() {
        let step = StepBuyCardsAndInducements::new();
        let roster_id = "human.lrb6";
        let pos_ids = bb2020_position_ids_for(roster_id);
        let mut game = make_game_bb2020(roster_id);
        step.add_mercenaries(&mut game, true, &[pos_ids[0].clone()], &[None]);
        assert_eq!(game.team_home.players.len(), 1);
        assert!(game.team_home.players[0].has_skill(SkillId::Loner),
            "mercenary must have Loner skill");
    }

    #[test]
    fn add_mercenaries_player_type_is_mercenary() {
        let step = StepBuyCardsAndInducements::new();
        let roster_id = "human.lrb6";
        let pos_ids = bb2020_position_ids_for(roster_id);
        let mut game = make_game_bb2020(roster_id);
        step.add_mercenaries(&mut game, true, &[pos_ids[0].clone()], &[None]);
        assert_eq!(game.team_home.players[0].player_type, PlayerType::Mercenary);
    }

    #[test]
    fn add_mercenaries_with_extra_skill_adds_it() {
        let step = StepBuyCardsAndInducements::new();
        let roster_id = "human.lrb6";
        let pos_ids = bb2020_position_ids_for(roster_id);
        let mut game = make_game_bb2020(roster_id);
        step.add_mercenaries(&mut game, true, &[pos_ids[0].clone()], &[Some(SkillId::Dodge)]);
        let p = &game.team_home.players[0];
        assert!(p.has_skill(SkillId::Loner), "must have Loner");
        assert!(p.has_skill(SkillId::Dodge), "must have extra skill Dodge");
    }

    #[test]
    fn add_mercenaries_extra_cost_option_applied() {
        use ffb_model::option::game_option_id::INDUCEMENT_MERCENARIES_EXTRA_COST;
        let roster_id = "human.lrb6";
        let pos_ids = bb2020_position_ids_for(roster_id);
        let mut game = make_game_bb2020(roster_id);
        game.options.set(INDUCEMENT_MERCENARIES_EXTRA_COST, "30000");
        let step = StepBuyCardsAndInducements::new();
        use ffb_model::data::loader::bb2020_rosters;
        let pos_cost = bb2020_rosters().into_iter()
            .find(|r| r.id == roster_id).unwrap()
            .positions.into_iter()
            .find(|p| p.id == pos_ids[0]).unwrap()
            .cost;
        let cost = step.add_mercenaries(&mut game, true, &[pos_ids[0].clone()], &[None]);
        assert_eq!(cost, pos_cost + 30_000);
    }

    // ── add_staff tests ────────────────────────────────────────────────────────

    #[test]
    fn add_staff_empty_returns_zero() {
        let step = StepBuyCardsAndInducements::new();
        let mut game = make_game();
        let cost = step.add_staff(&mut game, true, vec![]);
        assert_eq!(cost, 0);
        assert_eq!(game.team_home.players.len(), 0);
    }

    #[test]
    fn add_staff_unknown_position_skipped() {
        let step = StepBuyCardsAndInducements::new();
        let mut game = make_game_bb2020("human.lrb6");
        let cost = step.add_staff(&mut game, true, vec!["NONEXISTENT".to_string()]);
        assert_eq!(cost, 0);
        assert_eq!(game.team_home.players.len(), 0);
    }

    #[test]
    fn add_staff_adds_infamous_staff_type() {
        let step = StepBuyCardsAndInducements::new();
        let roster_id = "human.lrb6";
        let pos_ids = bb2020_position_ids_for(roster_id);
        let mut game = make_game_bb2020(roster_id);
        step.add_staff(&mut game, true, vec![pos_ids[0].clone()]);
        assert_eq!(game.team_home.players.len(), 1);
        assert_eq!(game.team_home.players[0].player_type, PlayerType::InfamousStaff);
    }

    #[test]
    fn add_staff_id_uses_team_prefix_i() {
        let step = StepBuyCardsAndInducements::new();
        let roster_id = "human.lrb6";
        let pos_ids = bb2020_position_ids_for(roster_id);
        let mut game = make_game_bb2020(roster_id);
        step.add_staff(&mut game, true, vec![pos_ids[0].clone()]);
        let player_id = &game.team_home.players[0].id;
        assert!(player_id.starts_with("home"), "id must start with team id");
        assert!(player_id.contains('I'), "id must contain 'I' for InfamousStaff");
    }

    // ── remove_duplicate_player_inducements tests ──────────────────────────────

    #[test]
    fn remove_duplicate_player_inducements_no_op_when_not_present() {
        let mut game = make_game();
        // Should not panic when no inducement with STAR usage is present.
        StepBuyCardsAndInducements::remove_duplicate_player_inducements(
            &mut game.turn_data_home, 1, Usage::STAR,
        );
    }
}
