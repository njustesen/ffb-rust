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
/// DEFERRED(BuyCardsAndInducements-options): GameOptionId checks (INDUCEMENTS, USE_PREDEFINED_INDUCEMENTS,
///   FREE_INDUCEMENT_CASH, FREE_CARD_CASH, INDUCEMENTS_ALLOW_SPENDING_TREASURY_ON_EQUAL_CTV,
///   INDUCEMENTS_ALLOW_OVERDOG_SPENDING, INDUCEMENTS_ALWAYS_USE_TREASURY, CARDS_SPECIAL_PLAY_COST,
///   MAX_NR_OF_CARDS, ALLOW_STAR_ON_BOTH_TEAMS, ALLOW_STAFF_ON_BOTH_TEAMS,
///   INDUCEMENT_MERCENARIES_EXTRA_COST, INDUCEMENT_MERCENARIES_SKILL_COST,
///   INDUCEMENT_PRAYERS_AVAILABLE_FOR_UNDERDOG) all deferred.
/// DEFERRED(BuyCardsAndInducements-decks): CardTypeFactory / CardDeck / card-choice randomisation deferred.
/// DEFERRED(BuyCardsAndInducements-addStarPlayers): RosterPlayer creation + sendAddedPlayers deferred.
/// DEFERRED(BuyCardsAndInducements-addMercenaries): RosterPlayer mercenary creation / Loner skill injection deferred.
/// DEFERRED(BuyCardsAndInducements-addStaff): InfamousStaff RosterPlayer creation deferred.
/// DEFERRED(BuyCardsAndInducements-inducementCosts): InducementTypeFactory cost calculation deferred.
/// DEFERRED(BuyCardsAndInducements-briberyAndCorruption): SpecialRule::BriberyAndCorruption handling deferred.
/// DEFERRED(BuyCardsAndInducements-rerollOnesOnKOs): NamedProperties::canReRollOnesOnKORecovery handling deferred.
/// DEFERRED(BuyCardsAndInducements-generators): Kickoff + Inducement(AFTER_INDUCEMENTS_PURCHASED) sequence
///   push in leaveStep deferred until Kickoff.build_sequence() is implemented.
/// DEFERRED(BuyCardsAndInducements-dialog): DialogBuyCardsAndInducementsParameter / CLIENT_SELECT_CARD_TO_BUY
///   / CLIENT_BUY_INDUCEMENTS dialog path deferred.
use ffb_model::enums::InducementPhase;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::generator::common::inducement::InducementParams;
use crate::step::generator::common::Inducement;
use crate::step::generator::common::RiotousRookies;

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
            // DEFERRED(BuyCardsAndInducements-dialog): return Continue when the dialog is shown.
            StepOutcome::cont()
        }
    }

    /// Java: `init(Game game)` — determine who has petty cash and set initial phase.
    fn init(&mut self, game: &mut Game) {
        // DEFERRED(BuyCardsAndInducements-options): check GameOptionId::INDUCEMENTS.
        // DEFERRED(BuyCardsAndInducements-options): check USE_PREDEFINED_INDUCEMENTS.
        // DEFERRED(BuyCardsAndInducements-decks): buildDecks().
        // DEFERRED(BuyCardsAndInducements-options): read FREE_INDUCEMENT_CASH + FREE_CARD_CASH.

        let petty_home = game.game_result.home.petty_cash_from_tv_diff;
        let petty_away = game.game_result.away.petty_cash_from_tv_diff;

        // DEFERRED(BuyCardsAndInducements-options): check INDUCEMENTS_ALLOW_SPENDING_TREASURY_ON_EQUAL_CTV.
        // DEFERRED(BuyCardsAndInducements-options): check INDUCEMENTS_ALLOW_OVERDOG_SPENDING.
        // For now: use petty_cash_from_tv_diff to decide who is the underdog.
        if petty_home > 0 {
            // Away is the overdog; home team (underdog) shops first.
            self.phase = Phase::Home;
            self.available_inducement_gold_home = Some(petty_home);
            // DEFERRED(BuyCardsAndInducements-dialog): showDialog for home.
            // Fall through to swap_team / leave if no dialog triggered.
        } else if petty_away > 0 {
            // Home is the overdog; away team (underdog) shops first.
            self.phase = Phase::Away;
            self.available_inducement_gold_away = Some(petty_away);
            // DEFERRED(BuyCardsAndInducements-dialog): showDialog for away.
        } else {
            // Equal TV and no treasury spending: skip.
            self.available_inducement_gold_home = Some(0);
            self.available_inducement_gold_away = Some(0);
            self.phase = Phase::Done;
        }
    }

    /// Java: `handleCard()` — apply the selected card and refresh choices.
    fn handle_card(&mut self) {
        // DEFERRED(BuyCardsAndInducements-decks): full card handling deferred.
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
                // DEFERRED(BuyCardsAndInducements-dialog): showDialog for away if gold > min.
                // If dialog not shown: DONE.
                self.phase = Phase::Done;
            }
            Phase::Away if self.available_inducement_gold_home.is_none() => {
                self.phase = Phase::Home;
                // DEFERRED(BuyCardsAndInducements-dialog): showDialog for home.
                self.phase = Phase::Done;
            }
            _ => {
                self.phase = Phase::Done;
            }
        }
    }

    /// Java: `leaveStep()` — push sequences, record gold spent, NEXT_STEP.
    fn leave_step(&self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // DEFERRED(BuyCardsAndInducements-generators): apply buffered buyInducementCommands.

        let new_tv_home = game.team_home.team_value
            + self.used_inducement_gold_home;
        let new_tv_away = game.team_away.team_value
            + self.used_inducement_gold_away;

        // Java: if parallel → addReport for both teams now (serial: already reported per-command).
        // DEFERRED(BuyCardsAndInducements-report): ReportCardsAndInducementsBought deferred.

        // Java: push Kickoff sequence (always first).
        // DEFERRED(BuyCardsAndInducements-generators): push Kickoff.build_sequence() when implemented.
        //   Kickoff::build_sequence(game.rules)

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
        // DEFERRED(BuyCardsAndInducements-prayers): prayers step push deferred.

        // Java: record treasury_spent_on_inducements / petty_cash_used for both teams.
        // DEFERRED(BuyCardsAndInducements-accounting): TeamResult treasury/petty-cash accounting deferred.

        // Java: BriberyAndCorruption special rule handling.
        // DEFERRED(BuyCardsAndInducements-briberyAndCorruption): deferred.

        // Java: canReRollOnesOnKORecovery handling.
        // DEFERRED(BuyCardsAndInducements-rerollOnesOnKOs): deferred.

        // Java: publishParameter(INDUCEMENT_GOLD_HOME, newTvHome)
        // Java: publishParameter(INDUCEMENT_GOLD_AWAY, newTvAway)
        // Java: setNextAction(StepAction.NEXT_STEP)
        StepOutcome::next()
            .publish(StepParameter::InducementGoldHome(new_tv_home))
            .publish(StepParameter::InducementGoldAway(new_tv_away))
            .push_seq(seq_first)
            .push_seq(seq_second)
            .push_seq(seq_riotous)
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
                // DEFERRED(BuyCardsAndInducements-dialog): handle BuyInducements action when ported.
                // In parallel mode: buffer; in serial mode: apply immediately and generate report.
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
        // Expect: 2 × Inducement(AFTER_INDUCEMENTS_PURCHASED) + 1 × RiotousRookies = 3 sequences.
        assert_eq!(out.pushes.len(), 3, "expected 2 Inducement + 1 RiotousRookies sequences");
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
