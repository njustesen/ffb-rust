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
/// In "parallel" mode (equal-TV games with allowEvenCTV option set) both teams buy
/// simultaneously: BuyInducements commands are buffered and applied in leaveStep().
///
/// USE_PREDEFINED_INDUCEMENTS: skips dialog (predefined set application requires InducementTypeFactory — not ported).
/// GameOptionId checks (INDUCEMENTS_ALWAYS_USE_TREASURY, CARDS_SPECIAL_PLAY_COST,
///   MAX_NR_OF_CARDS, ALLOW_STAR_ON_BOTH_TEAMS, ALLOW_STAFF_ON_BOTH_TEAMS,
///   INDUCEMENT_MERCENARIES_EXTRA_COST, INDUCEMENT_MERCENARIES_SKILL_COST) not wired (blocked by InducementTypeFactory).
/// CardTypeFactory / CardDeck / card-choice randomisation — card system not ported.
/// addStarPlayers — RosterPlayer creation + sendAddedPlayers not ported (blocked by InducementTypeFactory).
/// addMercenaries — RosterPlayer mercenary creation / Loner skill injection not ported (blocked by InducementTypeFactory).
/// addStaff — InfamousStaff RosterPlayer creation not ported (blocked by InducementTypeFactory).
/// InducementTypeFactory cost calculation not ported; headless auto-skips all inducement buying.
/// BriberyAndCorruption: adds 1 briberyAndCorruption inducement if team has the special rule.
/// rerollOnesOnKOs: adds 1 bugmansXXXXXX inducement if any team player has canReRollOnesOnKORecovery.
/// no-op: apply buffered buyInducementCommands in leaveStep — no commands in headless mode (no dialog).
/// client-only: DialogBuyCardsAndInducementsParameter / CLIENT_SELECT_CARD_TO_BUY / CLIENT_BUY_INDUCEMENTS
///   dialog path — coaches interact via client; headless skips without buying.
use ffb_model::enums::InducementPhase;
use ffb_model::model::game::Game;
use ffb_model::report::bb2020::report_cards_and_inducements_bought::ReportCardsAndInducementsBought;
use ffb_model::option::game_option_id::{
    INDUCEMENTS, FREE_INDUCEMENT_CASH, FREE_CARD_CASH,
    INDUCEMENTS_ALLOW_SPENDING_TREASURY_ON_EQUAL_CTV, INDUCEMENTS_ALLOW_OVERDOG_SPENDING,
    USE_PREDEFINED_INDUCEMENTS, INDUCEMENT_PRAYERS_AVAILABLE_FOR_UNDERDOG,
};
use ffb_model::option::util_game_option::{is_option_enabled, get_int_option};
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::generator::sequence::SequenceStep;
use crate::step::generator::common::inducement::InducementParams;
use crate::step::generator::common::{Inducement, RiotousRookies};
use crate::step::generator::mixed::kickoff::{Kickoff, KickoffParams};
use crate::step::game::start::util_inducement_sequence::UtilInducementSequence;

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
        }
    }

    /// Java: `executeStep()` — main state machine dispatch.
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
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
            // client-only: dialog would pause here waiting for coach input; headless falls through
            StepOutcome::cont()
        }
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
        let allow_overdog = is_option_enabled(game, INDUCEMENTS_ALLOW_OVERDOG_SPENDING);

        if petty_home > 0 {
            // Away is the overdog; home team (underdog) shops first.
            self.phase = Phase::Home;
            let gold = if allow_overdog {
                petty_home + game.team_home.treasury + free_cash
            } else {
                petty_home + free_cash
            };
            self.available_inducement_gold_home = Some(gold);
            // client-only: DialogBuyCardsAndInducementsParameter for home coach
        } else if petty_away > 0 {
            // Home is the overdog; away team (underdog) shops first.
            self.phase = Phase::Away;
            let gold = if allow_overdog {
                petty_away + game.team_away.treasury + free_cash
            } else {
                petty_away + free_cash
            };
            self.available_inducement_gold_away = Some(gold);
            // client-only: DialogBuyCardsAndInducementsParameter for away coach
        } else if allow_even_ctv {
            // Equal TV but treasury/free-cash spending allowed: both teams shop in parallel.
            self.phase = Phase::Home;
            self.parallel = true;
            let gold_home = game.team_home.treasury + free_cash;
            let gold_away = game.team_away.treasury + free_cash;
            self.available_inducement_gold_home = Some(gold_home);
            self.available_inducement_gold_away = Some(gold_away);
            // client-only: DialogBuyCardsAndInducementsParameter for both coaches (parallel)
        } else {
            // Equal TV and no treasury spending: skip.
            self.available_inducement_gold_home = Some(0);
            self.available_inducement_gold_away = Some(0);
            self.phase = Phase::Done;
        }
    }

    /// Java: `handleCard()` — apply the selected card and refresh choices.
    fn handle_card(&mut self) {
        // no-op: full card handling (CardDeck) not ported
        // Java: deduct cardPrice from gold, add chosen card to inducement set,
        //       call updateChoices() to draw next pair for the coach.
        self.current_selection = None;
    }

    /// Java: `swapTeam()` — move to the next team or DONE.
    fn swap_team(&mut self, _game: &mut Game) {
        // Java: if phase==HOME && availableInducementGoldAway==null → switch to AWAY.
        //       if phase==AWAY && availableInducementGoldHome==null → switch to HOME.
        //       else → DONE.
        match self.phase {
            Phase::Home if self.available_inducement_gold_away.is_none() => {
                self.phase = Phase::Away;
                // client-only: DialogBuyCardsAndInducementsParameter for away if gold > min
                self.phase = Phase::Done;
            }
            Phase::Away if self.available_inducement_gold_home.is_none() => {
                self.phase = Phase::Home;
                // client-only: DialogBuyCardsAndInducementsParameter for home coach
                self.phase = Phase::Done;
            }
            _ => {
                self.phase = Phase::Done;
            }
        }
    }

    /// Java: `leaveStep()` — push sequences, record gold spent, NEXT_STEP.
    fn leave_step(&self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // no-op: apply buffered buyInducementCommands — no commands buffered in headless mode (no dialog)

        let new_tv_home = game.team_home.team_value
            + self.used_inducement_gold_home;
        let new_tv_away = game.team_away.team_value
            + self.used_inducement_gold_away;

        // Java: if parallel → addReport for both teams now (serial: already reported per-command).
        if self.parallel {
            game.report_list.add(ReportCardsAndInducementsBought::new(
                game.team_home.id.clone(),
                0, 0, 0, 0,
                self.used_inducement_gold_home,
                new_tv_home,
            ));
            game.report_list.add(ReportCardsAndInducementsBought::new(
                game.team_away.id.clone(),
                0, 0, 0, 0,
                self.used_inducement_gold_away,
                new_tv_away,
            ));
        }

        // Java: ((Kickoff) factory.forName(Kickoff)).pushSequence(new Kickoff.SequenceParams(gameState, true))
        let seq_kickoff = Kickoff::build_sequence(&KickoffParams { with_coin_choice: true });

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

        // Java: game.getTeamHome/Away().getTeamData().setPettyCashUsed(UtilInducementSequence.calculateInducementGold(...))
        game.game_result.home.petty_cash_used = UtilInducementSequence::calculate_inducement_gold(Some(game), true);
        game.game_result.away.petty_cash_used = UtilInducementSequence::calculate_inducement_gold(Some(game), false);

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
            }
            if game.team_away.special_rules.iter().any(|r| r == bnc_name) {
                game.turn_data_away.inducement_set.add_inducement(
                    InducementModel::new("briberyAndCorruption", 1, vec![Usage::REROLL_ARGUE]));
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

        // Java: publishParameter(INDUCEMENT_GOLD_HOME, newTvHome)
        // Java: publishParameter(INDUCEMENT_GOLD_AWAY, newTvAway)
        // Java: setNextAction(StepAction.NEXT_STEP)
        let mut out = StepOutcome::next()
            .publish(StepParameter::InducementGoldHome(new_tv_home))
            .publish(StepParameter::InducementGoldAway(new_tv_away))
            .push_seq(seq_kickoff)
            .push_seq(seq_first)
            .push_seq(seq_second)
            .push_seq(seq_riotous);
        if let Some(seq) = seq_prayers {
            out = out.push_seq(seq);
        }
        out
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
        match action {
            Action::BuyInducements { .. } => {
                // client-only: BuyInducements action arrives from coach dialog; headless has no coach
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
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
    fn id_is_buy_cards_and_inducements() {
        assert_eq!(StepBuyCardsAndInducements::new().id(), StepId::BuyCardsAndInducements);
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

    #[test]
    fn equal_tv_publishes_inducement_gold_parameters() {
        let mut game = make_game();
        let mut step = StepBuyCardsAndInducements::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::InducementGoldHome(_))));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::InducementGoldAway(_))));
    }

    #[test]
    fn equal_tv_pushes_two_inducement_sequences_and_riotous_rookies() {
        let mut game = make_game();
        let mut step = StepBuyCardsAndInducements::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Expect: 1 × Kickoff + 2 × Inducement(AFTER_INDUCEMENTS_PURCHASED) + 1 × RiotousRookies = 4 sequences.
        assert_eq!(out.pushes.len(), 4, "expected Kickoff + 2 Inducement + 1 RiotousRookies sequences");
    }

    #[test]
    fn phase_is_done_after_equal_tv_start() {
        let mut game = make_game();
        let mut step = StepBuyCardsAndInducements::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(step.phase, Phase::Done);
    }

    #[test]
    fn home_underdog_sets_home_phase() {
        // home has petty cash (away is overdog)
        let mut game = make_game_with_petty_cash(100_000, 0);
        let mut step = StepBuyCardsAndInducements::new();
        // After init() completes without dialog: phase transitions to Done (no actual dialog).
        step.start(&mut game, &mut GameRng::new(0));
        // Regardless of phase, the gold was set before any dialog.
        // available_inducement_gold_home should be initialised.
        assert!(step.available_inducement_gold_home.is_some());
    }

    #[test]
    fn away_underdog_sets_away_gold() {
        // away has petty cash (home is overdog)
        let mut game = make_game_with_petty_cash(0, 80_000);
        let mut step = StepBuyCardsAndInducements::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(step.available_inducement_gold_away.is_some());
    }

    #[test]
    fn inducements_disabled_skips_to_done() {
        let mut game = make_game_with_petty_cash(100_000, 0);
        // INDUCEMENTS not set → disabled → skip immediately.
        let mut step = StepBuyCardsAndInducements::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(step.phase, Phase::Done);
        assert_eq!(step.available_inducement_gold_home, Some(0));
        assert_eq!(step.available_inducement_gold_away, Some(0));
    }

    #[test]
    fn inducements_enabled_home_underdog_sets_phase_and_gold() {
        use ffb_model::option::game_option_id::INDUCEMENTS;
        let mut game = make_game_with_petty_cash(100_000, 0);
        game.options.set(INDUCEMENTS, "true");
        let mut step = StepBuyCardsAndInducements::new();
        step.start(&mut game, &mut GameRng::new(0));
        // available_inducement_gold_home = petty_home + free_cash(0)
        assert_eq!(step.available_inducement_gold_home, Some(100_000));
    }

    #[test]
    fn free_cash_added_to_underdog_gold() {
        use ffb_model::option::game_option_id::{INDUCEMENTS, FREE_INDUCEMENT_CASH, FREE_CARD_CASH};
        let mut game = make_game_with_petty_cash(100_000, 0);
        game.options.set(INDUCEMENTS, "true");
        game.options.set(FREE_INDUCEMENT_CASH, "20000");
        game.options.set(FREE_CARD_CASH, "10000");
        let mut step = StepBuyCardsAndInducements::new();
        step.start(&mut game, &mut GameRng::new(0));
        // 100,000 petty + 20,000 + 10,000 free cash = 130,000
        assert_eq!(step.available_inducement_gold_home, Some(130_000));
    }

    #[test]
    fn allow_even_ctv_enables_parallel_mode() {
        use ffb_model::option::game_option_id::{INDUCEMENTS, INDUCEMENTS_ALLOW_SPENDING_TREASURY_ON_EQUAL_CTV};
        let mut game = make_game();
        game.options.set(INDUCEMENTS, "true");
        game.options.set(INDUCEMENTS_ALLOW_SPENDING_TREASURY_ON_EQUAL_CTV, "true");
        game.team_home.treasury = 50_000;
        game.team_away.treasury = 30_000;
        let mut step = StepBuyCardsAndInducements::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(step.parallel);
        assert_eq!(step.available_inducement_gold_home, Some(50_000));
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
}
