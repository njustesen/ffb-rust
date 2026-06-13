//! Step engine: the `Step` enum (concrete steps + dispatch), the `StepStack`, and the
//! `GameState` driver loop — the Rust port of Java `GameState.executeStep`.
//! See `docs/step_port/00_framework.md` (driver) and `10_sequences.md` (sequences).
//!
//! Steps are dispatched via the `Step` enum (no `dyn`). A step's `start`/`handle_command`
//! return a `StepOutcome` (next action + events + sub-sequences to push + params to publish +
//! an optional prompt). The driver — sole owner of the stack — applies the pushes and processes
//! the action, so a step only ever borrows `&mut Game` + `&mut GameRng`.
//!
//! Boundary (Java `ClientCommand`/`DialogParameter` analogue): the engine speaks the harness's
//! `Action`/`AgentPrompt` vocabulary directly. A step that must wait yields `Continue` + a
//! `prompt`; the driver surfaces it via `current_prompt()`; the harness's agent answers with an
//! `Action`, which `apply()` feeds to the waiting step's `handle_command`. (The wire
//! `ClientCommand` mapping is the networking phase, G/I; the engine/parity path uses `Action`.)

use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::events::GameEvent;
use ffb_model::enums::{GameStatus, Weather};
use ffb_model::prompts::AgentPrompt;

use ffb_model::model::team::Team;
use ffb_model::enums::{Rules, Direction, KickoffResult, PlayerState, PS_STANDING, PS_RESERVE};
use ffb_model::types::{FieldCoordinate, FIELD_WIDTH, FIELD_HEIGHT};
use ffb_model::kickoff::{kickoff_event_bb2025, KickoffEventKind};
use ffb_model::util::state_hash::state_hash;
use ffb_mechanics::mechanics::scatter_coordinate;

use crate::action::Action;
use crate::legal_actions::TeamSide;
use super::framework::{StepAction, StepId, StepParameter};

/// Place a team's available (RESERVE / unset) players in the canonical parity formation —
/// 1:1 with Java `ParityRunner.placeReserves()` (and the validated monolith port): jersey
/// order, ≤11, first three on the LOS (x=12), then the overflow squares; away mirrored x→25-x.
fn place_team_canonical(game: &mut Game, home: bool) {
    let los: &[(i32, i32)] = &[(12, 7), (12, 6), (12, 8), (12, 5), (12, 9), (12, 4), (12, 10)];
    let overflow: &[(i32, i32)] = &[
        (5, 5), (5, 7), (5, 9), (6, 6), (6, 8), (4, 6), (4, 8), (3, 6), (3, 8), (2, 5), (2, 9), (1, 7),
    ];
    let mut players: Vec<(String, i32)> = if home {
        game.team_home.players.iter().map(|p| (p.id.clone(), p.nr)).collect()
    } else {
        game.team_away.players.iter().map(|p| (p.id.clone(), p.nr)).collect()
    };
    players.sort_by_key(|&(_, nr)| nr);
    players.truncate(11);
    let available: Vec<&(String, i32)> = players.iter().filter(|(pid, _)| {
        match game.field_model.player_state(pid) {
            None => true,                          // unset before first setup = available
            Some(s) => s.base() == PS_RESERVE,
        }
    }).collect();
    let los_needed = available.len().min(3);
    for (placed, (pid, _)) in available.iter().enumerate() {
        let (ox, oy) = if placed < los_needed {
            los[placed]
        } else {
            let i = placed - los_needed;
            if i < overflow.len() { overflow[i] } else { continue }
        };
        let coord = if home { FieldCoordinate::new(ox, oy) } else { FieldCoordinate::new(25 - ox, oy) };
        game.field_model.set_player_coordinate(pid, coord);
        game.field_model.set_player_state(pid, PlayerState::new(PS_STANDING));
    }
}

/// Map the rolled kickoff-event kind to the `KickoffResult` carried by events/params.
/// (The two enums mirror each other; this is the BB2025-reachable set.)
fn kickoff_result_from_kind(kind: KickoffEventKind) -> KickoffResult {
    match kind {
        KickoffEventKind::GetTheRef => KickoffResult::GetTheRef,
        KickoffEventKind::HighKick => KickoffResult::HighKick,
        KickoffEventKind::CheeringFans => KickoffResult::CheeringFans,
        KickoffEventKind::WeatherChange => KickoffResult::WeatherChange,
        KickoffEventKind::BrilliantCoaching => KickoffResult::BrilliantCoaching,
        KickoffEventKind::QuickSnap => KickoffResult::QuickSnap,
        KickoffEventKind::PitchInvasion => KickoffResult::PitchInvasion,
        KickoffEventKind::Riot => KickoffResult::Riot,
        KickoffEventKind::PerfectDefence => KickoffResult::PerfectDefence,
        KickoffEventKind::ThrowARock => KickoffResult::ThrowARock,
        KickoffEventKind::TimeOut => KickoffResult::TimeOut,
        KickoffEventKind::SolidDefence => KickoffResult::SolidDefence,
        KickoffEventKind::OficiousRef => KickoffResult::OficiousRef,
        KickoffEventKind::Blitz => KickoffResult::Blitz,
        KickoffEventKind::Charge => KickoffResult::Charge,
        KickoffEventKind::DodgySnack => KickoffResult::DodgySnack,
    }
}

// ── Concrete steps ──────────────────────────────────────────────────────────────
// One variant per ported step (BB2025 lineman set grows here per docs/step_port/20_steps).
// Each carries its persistent fields; pregame steps are stateless.

#[derive(Debug, Clone)]
pub enum Step {
    /// Game-start bookkeeping: mark the game active. (Java `StepInitStartGame`.)
    InitStartGame,
    /// Roll fan factor for home then away. (Java `mixed/start/StepSpectators`.)
    Spectators,
    /// Roll initial weather (2d6). (Java `game/start/StepWeather`.)
    Weather,
    /// Prompt the coin guess, then flip the coin (d2). (Java `bb2016/start/StepCoinChoice`.)
    CoinChoice,
    /// Prompt the coin winner to receive or kick; set first-offense. (Java `StepReceiveChoice`.)
    ReceiveChoice,
    /// First-kickoff bookkeeping: StartHalf. (Java `StepInitKickoff`.) 0 dice.
    InitKickoff,
    /// Canonical placement of the active team, then flip. (Java `StepSetup` ×2.) 0 dice.
    Setup,
    /// Latch the kick target: place the ball on the receiving half. (Java `StepKickoff`/KickBall.)
    Kickoff,
    /// Scatter the kicked ball: d8 direction + d6 distance. (Java `StepKickoffScatterRoll`.)
    KickoffScatterRoll,
    /// Roll the 2d6 kickoff-event table; publish the result. (Java `StepKickoffResultRoll`.)
    KickoffResultRoll,
}

impl Step {
    pub fn id(&self) -> StepId {
        match self {
            Step::InitStartGame => StepId::InitStartGame,
            Step::Spectators => StepId::Spectators,
            Step::Weather => StepId::Weather,
            Step::CoinChoice => StepId::CoinChoice,
            Step::ReceiveChoice => StepId::ReceiveChoice,
            Step::InitKickoff => StepId::Kickoff,
            Step::Setup => StepId::Setup,
            Step::Kickoff => StepId::Kickoff,
            Step::KickoffScatterRoll => StepId::KickoffScatterRoll,
            Step::KickoffResultRoll => StepId::KickoffResultRoll,
        }
    }

    /// The step's `start()` body (Java `AbstractStep.start`). Steps that complete immediately
    /// advance with `NextStep`; steps that wait for an agent decision return `Continue` + a
    /// `prompt` and do their work in `handle_command`.
    fn start(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match self {
            Step::InitStartGame => {
                game.status = GameStatus::Active;
                StepOutcome::next()
            }
            // Java StepSpectators: rollFanFactor() d3 home then d3 away; fanFactor = dedicatedFans
            // + roll. No GameEvent (not in the state hash).
            Step::Spectators => {
                let roll_home = rng.d3();
                game.team_home.fan_factor = game.team_home.dedicated_fans + roll_home;
                let roll_away = rng.d3();
                game.team_away.fan_factor = game.team_away.dedicated_fans + roll_away;
                StepOutcome::next()
            }
            // Java StepWeather: rollWeather() = 2d6, mapped by interpretRollWeather.
            Step::Weather => {
                let w1 = rng.d6();
                let w2 = rng.d6();
                let weather = Weather::for_roll(w1 + w2);
                game.weather = weather;
                StepOutcome::next().with_event(GameEvent::WeatherChange { weather })
            }
            // Java StepCoinChoice: the away coach guesses; the home prompt asks for the guess.
            // No dice in start — the coin is flipped in handle_command after the guess arrives.
            Step::CoinChoice => StepOutcome::cont().with_prompt(AgentPrompt::CoinChoice { is_home: true }),
            // Java StepReceiveChoice: the coin winner (now `home_playing`) chooses receive/kick.
            Step::ReceiveChoice => {
                let team_id = game.active_team().id.clone();
                StepOutcome::cont().with_prompt(AgentPrompt::ReceiveChoice { team_id })
            }
            // Java StepInitKickoff (first kickoff): start half 1. Bookkeeping, 0 dice.
            Step::InitKickoff => {
                StepOutcome::next().with_event(GameEvent::StartHalf { half: game.half })
            }
            // Java StepSetup ×2 (kicking then receiving team). The parity agent places its team
            // in the canonical formation; we place the active team and flip so the next Setup
            // handles the other. 0 dice, no prompt.
            Step::Setup => {
                place_team_canonical(game, game.home_playing);
                game.home_playing = !game.home_playing;
                StepOutcome::next()
            }
            // Java StepKickoff: latch the kick target and place the ball. The kicking team
            // (current `home_playing` after the two setups) kicks into the receiving half.
            // NOTE: provisional canonical target — the agent `KickBall` command replaces this
            // when chasing the state hash; the scatter dice below are independent of the target.
            Step::Kickoff => {
                let kicker_home = game.home_playing;
                // Receiving half is the opponent's: away half (x≥13) if home kicks, else home half.
                let target = if kicker_home { FieldCoordinate::new(21, 9) } else { FieldCoordinate::new(4, 9) };
                game.field_model.ball_coordinate = Some(target);
                StepOutcome::next()
            }
            // Java StepKickoffScatterRoll: scatter the kicked ball. Dice IN ORDER: d8 direction,
            // then d6 distance. Walk the landing back toward the start until on-pitch (Java's
            // findScatterCoordinate `lastValid` back-walk); a never-on-pitch scatter = touchback.
            // The ball's current square is the kick target (StepKickoff placed it).
            Step::KickoffScatterRoll => {
                let start = game.field_model.ball_coordinate.unwrap_or(FieldCoordinate::new(0, 0));
                let dir_roll = rng.d8();
                let direction = Direction::for_roll(dir_roll).expect("d8 roll is 1..=8");
                let distance = rng.d6();
                // Back-walk: full distance first, decrement until the square is on-pitch.
                let mut landing = None;
                let mut d = distance;
                while d > 0 {
                    let (x, y) = scatter_coordinate(start.x, start.y, direction, d);
                    let c = FieldCoordinate::new(x, y);
                    if x >= 0 && x < FIELD_WIDTH && y >= 0 && y < FIELD_HEIGHT {
                        landing = Some(c);
                        break;
                    }
                    d -= 1;
                }
                // touchback (landing == None) is resolved by the later Touchback step; for now
                // we only place the ball when it lands on-pitch.
                if let Some(c) = landing {
                    game.field_model.ball_coordinate = Some(c);
                }
                StepOutcome::next()
                    .with_event(GameEvent::KickoffScatter { start, direction: dir_roll, distance })
            }
            // Java StepKickoffResultRoll: rollKickoff() = 2d6; interpret → KickoffResult; publish it.
            Step::KickoffResultRoll => {
                let d1 = rng.d6();
                let d2 = rng.d6();
                let total = d1 + d2;
                let mut out = StepOutcome::next();
                if let Some(kind) = kickoff_event_bb2025(total) {
                    let result = kickoff_result_from_kind(kind);
                    out = out
                        .with_event(GameEvent::KickoffResultEvent { result })
                        .publish(StepParameter::KickoffResult(result));
                }
                out
            }
        }
    }

    /// The step's `handle_command()` body (Java `AbstractStep.handleCommand`). Called by the
    /// driver when an `Action` arrives for the waiting current step.
    fn handle_command(&self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match (self, action) {
            // Flip the coin with the game RNG (Java throwCoin = 1× d2). Winner = guess == coin.
            // The winner becomes `home_playing` (the chooser) for the following ReceiveChoice.
            (Step::CoinChoice, Action::CoinChoice { heads }) => {
                let coin_is_heads = rng.bool();
                let home_won = *heads == coin_is_heads;
                game.home_playing = home_won;
                StepOutcome::next().with_event(GameEvent::CoinThrow { home_won })
            }
            // The chooser's `receive` resolves to whether HOME has first offense. The KICKER
            // sets up first, so home_playing = !home_receives (matches Java setup ordering).
            (Step::ReceiveChoice, Action::ReceiveChoice { receive }) => {
                let home_receives = if game.home_playing { *receive } else { !*receive };
                game.home_first_offense = home_receives;
                game.home_playing = !home_receives;
                let team_id = if home_receives { game.team_home.id.clone() } else { game.team_away.id.clone() };
                StepOutcome::next().with_event(GameEvent::ReceiveChoice { team_id, receive: home_receives })
            }
            // A command the current step does not recognise (Java StepCommandStatus::UNHANDLED):
            // stay put and keep waiting. (The harness never sends one in the parity path.)
            _ => StepOutcome::cont(),
        }
    }

    /// Offer a published parameter to this step while the driver walks the stack top→bottom.
    /// Return `true` to consume it (stops propagation). Java `AbstractStep.setParameter`.
    /// Plumbing in place; the first consumers land with the Phase D steps that read params
    /// (e.g. MoveStack, EndTurn) — pregame steps consume nothing.
    fn set_parameter(&mut self, _param: &StepParameter) -> bool {
        false
    }
}

// ── Step outcome / stack ─────────────────────────────────────────────────────────

/// What a step produced: how to advance, the events it emitted, sub-sequences to push, params
/// to publish down the stack, and an optional prompt (when it yields `Continue` to wait).
pub struct StepOutcome {
    pub action: StepAction,
    pub goto_label: Option<String>,
    pub events: Vec<GameEvent>,
    /// Sequences to push (authored order; the stack reverses them on push).
    pub pushes: Vec<Vec<StepEntry>>,
    /// Parameters to publish down the stack (top→bottom) after this step runs.
    pub published: Vec<StepParameter>,
    /// Set together with `Continue` when the step is waiting for an agent decision.
    pub prompt: Option<AgentPrompt>,
}

impl StepOutcome {
    fn base(action: StepAction) -> Self {
        StepOutcome { action, goto_label: None, events: Vec::new(), pushes: Vec::new(), published: Vec::new(), prompt: None }
    }
    pub fn next() -> Self { Self::base(StepAction::NextStep) }
    pub fn cont() -> Self { Self::base(StepAction::Continue) }
    pub fn goto(label: &str) -> Self {
        let mut o = Self::base(StepAction::GotoLabel);
        o.goto_label = Some(label.to_owned());
        o
    }
    pub fn with_event(mut self, e: GameEvent) -> Self { self.events.push(e); self }
    pub fn with_prompt(mut self, p: AgentPrompt) -> Self { self.prompt = Some(p); self }
    pub fn push_seq(mut self, seq: Vec<StepEntry>) -> Self { self.pushes.push(seq); self }
    pub fn publish(mut self, p: StepParameter) -> Self { self.published.push(p); self }
}

/// A stacked step: the concrete step plus an optional label (goto target).
#[derive(Debug, Clone)]
pub struct StepEntry {
    pub step: Step,
    pub label: Option<String>,
}

impl StepEntry {
    pub fn new(step: Step) -> Self { StepEntry { step, label: None } }
    pub fn labelled(step: Step, label: &str) -> Self { StepEntry { step, label: Some(label.to_owned()) } }
    pub fn id(&self) -> StepId { self.step.id() }
}

/// LIFO step stack. Java keeps top at index 0; here top = last (`Vec::last`).
/// `push_sequence` pushes authored order REVERSED so the first-authored step ends on top
/// and runs first (matches Java's back-to-front push at index 0).
#[derive(Default)]
pub struct StepStack {
    steps: Vec<StepEntry>,
}

impl StepStack {
    pub fn new() -> Self { Self::default() }
    pub fn push(&mut self, step: StepEntry) { self.steps.push(step); }
    pub fn push_sequence(&mut self, seq: Vec<StepEntry>) {
        for s in seq.into_iter().rev() { self.steps.push(s); }
    }
    pub fn pop(&mut self) -> Option<StepEntry> { self.steps.pop() }
    pub fn peek(&self) -> Option<&StepEntry> { self.steps.last() }
    pub fn len(&self) -> usize { self.steps.len() }
    pub fn is_empty(&self) -> bool { self.steps.is_empty() }

    /// Pop the stack down until the labelled step is on top (left in place). Java
    /// `handleStepResultGotoLabel`: discard intervening steps; error if the label is absent.
    pub fn goto_label(&mut self, label: &str) -> Result<(), String> {
        while let Some(top) = self.steps.last() {
            if top.label.as_deref() == Some(label) {
                return Ok(());
            }
            self.steps.pop();
        }
        Err(format!("goto unknown label '{label}'"))
    }

    /// Publish a parameter down the stack (top→bottom), stopping once a step consumes it.
    /// Java `StepStack.publishParameter` → each step's `setParameter`. The publisher is the
    /// current step (already popped into the driver), so this only reaches steps below it.
    pub fn publish(&mut self, param: &StepParameter) {
        for entry in self.steps.iter_mut().rev() {
            if entry.step.set_parameter(param) {
                return;
            }
        }
    }
}

// ── Driver ──────────────────────────────────────────────────────────────────────

/// The game driver — owns the model, RNG, step stack and current step. Port of Java
/// `GameState` (the executeStep/processStepResult loop, flattened to an explicit loop per
/// `00_framework.md` §7). Drives start-mode chains and command-mode (handle_command) steps,
/// surfacing an `AgentPrompt` when a step waits and accepting an `Action` to resume.
pub struct GameState {
    pub game: Game,
    pub rng: GameRng,
    stack: StepStack,
    current: Option<StepEntry>,
    /// When `Some`, the next drive of `current` re-delivers this command (NextStep/GotoLabel
    /// *AndRepeat* — Java's `forwardCommand`) instead of calling `start`.
    forwarded: Option<Action>,
    /// The prompt the waiting current step raised; `None` when the engine is idle.
    pending_prompt: Option<AgentPrompt>,
    /// Events accumulated since the last drain (the parity log reads these).
    pub events: Vec<GameEvent>,
    /// State hash of the FRESH game, captured before any roll — the parity log's GameStart
    /// (i:0) hash. Java logs this on the freshly-created game, so we snapshot it in `new`
    /// before the StartGame sequence runs.
    initial_hash: String,
}

impl GameState {
    /// Construct directly from a pre-built `Game` (used by step characterization tests; the
    /// caller pushes a sequence and drives explicitly).
    pub fn from_game(game: Game, seed: u64) -> Self {
        GameState {
            game, rng: GameRng::new(seed), stack: StepStack::new(),
            current: None, forwarded: None, pending_prompt: None, events: Vec::new(),
            initial_hash: String::new(),
        }
    }

    /// Game-driver entry point the parity harness constructs from: build the game, snapshot the
    /// fresh-game (pre-roll) GameStart hash, push the StartGame sequence, and run to the first
    /// prompt so `current_prompt()` is immediately available.
    pub fn new(home: Team, away: Team, rules: Rules, seed: u64) -> Self {
        let game = Game::new(home, away, rules);
        let mut gs = GameState::from_game(game, seed);
        gs.initial_hash = state_hash(&gs.game); // fresh, before any StartGame roll
        gs.push_sequence(start_game_sequence());
        gs.run_until_prompt();
        gs
    }

    /// The GameStart (i:0) state hash — the fresh game before any roll. (Parity log anchor.)
    pub fn initial_state_hash(&self) -> &str { &self.initial_hash }

    pub fn push_sequence(&mut self, seq: Vec<StepEntry>) { self.stack.push_sequence(seq); }

    /// The prompt the engine is currently waiting on, if any. `None` ⇒ idle (stack drained).
    pub fn current_prompt(&self) -> Option<&AgentPrompt> { self.pending_prompt.as_ref() }

    /// Drain events accumulated so far, resetting the buffer (parity log read point).
    pub fn take_events(&mut self) -> Vec<GameEvent> { std::mem::take(&mut self.events) }

    // ── Harness-facing facade ──────────────────────────────────────────────────────
    // The parity harness is engine-agnostic: it needs only these few methods + `.game`.

    /// The side currently to act (derived from the model — the engine infers it, so `apply`'s
    /// `side` argument is advisory only).
    pub fn active_side(&self) -> TeamSide {
        if self.game.home_playing { TeamSide::Home } else { TeamSide::Away }
    }

    /// Whether the game has ended.
    pub fn is_finished(&self) -> bool { self.game.is_finished() }

    /// Game-dice draw count (parity diagnostics / no-progress guard).
    pub fn rng_call_count(&self) -> u64 { self.rng.call_count }

    /// FNV-1a 64-bit state hash (matches Java's `ParityRunner.stateHash()`).
    pub fn state_hash_str(&self) -> String { state_hash(&self.game) }

    /// Feed an agent decision and advance, returning the events produced. The `side` is advisory
    /// (the engine infers the acting side); kept for the harness's call shape.
    pub fn apply(&mut self, _side: TeamSide, action: Action) -> Result<Vec<GameEvent>, String> {
        self.apply_action(action);
        Ok(self.take_events())
    }

    /// Apply a step's side effects to driver-owned state (events, sub-sequence pushes, and
    /// published parameters). Shared by start- and command-mode.
    fn apply_effects(&mut self, outcome: &mut StepOutcome) {
        self.events.append(&mut outcome.events);
        for seq in outcome.pushes.drain(..) { self.stack.push_sequence(seq); }
        for param in outcome.published.drain(..) { self.stack.publish(&param); }
    }

    /// Feed an agent decision to the waiting current step (Java command-mode `executeStep`),
    /// then drive forward until the next prompt or idle. Internal driver entry; the harness
    /// uses the `apply(side, action)` facade above.
    pub fn apply_action(&mut self, action: Action) {
        let entry = self.current.take().expect("apply_action() with no waiting step");
        let mut outcome = entry.step.handle_command(&action, &mut self.game, &mut self.rng);
        self.apply_effects(&mut outcome);
        self.pending_prompt = None;
        self.dispatch(entry, action, outcome);
        self.drive();
    }

    /// Drive the start-mode chain until a step waits (Continue + prompt) or the stack empties.
    /// Mirrors `GameState.executeStep`'s start-mode loop + `processStepResult`.
    pub fn run_until_prompt(&mut self) { self.drive(); }

    fn drive(&mut self) {
        loop {
            // Already waiting on a prompt from a prior apply/dispatch — nothing to start.
            if self.current.is_some() && self.pending_prompt.is_some() {
                return;
            }
            if self.current.is_none() {
                match self.stack.pop() {
                    Some(s) => self.current = Some(s),
                    None => { self.pending_prompt = None; return; }
                }
            }
            let entry = self.current.take().unwrap();
            // Forwarded command (AndRepeat) → re-deliver via handle_command; else start().
            let mut outcome = match self.forwarded.take() {
                Some(cmd) => {
                    let o = entry.step.handle_command(&cmd, &mut self.game, &mut self.rng);
                    // keep cmd available in case this step also forwards
                    self.dispatch(entry, cmd, o);
                    if self.pending_prompt.is_some() { return; }
                    continue;
                }
                None => entry.step.start(&mut self.game, &mut self.rng),
            };
            self.apply_effects(&mut outcome);
            match outcome.action {
                StepAction::Continue | StepAction::Repeat => {
                    // Continue: wait for a command (prompt set by the step). Repeat: pregame
                    // steps don't use it; treated as idle until a repeat()-capable step lands.
                    self.pending_prompt = outcome.prompt;
                    self.current = Some(entry);
                    return;
                }
                StepAction::NextStep => { self.current = None; }
                StepAction::GotoLabel => {
                    let label = outcome.goto_label.expect("goto without label");
                    self.stack.goto_label(&label).expect("goto label present");
                    self.current = None;
                }
                StepAction::NextStepAndRepeat | StepAction::GotoLabelAndRepeat => {
                    // forwardCommand from a start() result has no command to forward; treat as
                    // the non-repeat variant. (Forwarding only originates from handle_command.)
                    if outcome.action.trigger_goto() {
                        let label = outcome.goto_label.expect("goto without label");
                        self.stack.goto_label(&label).expect("goto label present");
                    }
                    self.current = None;
                }
            }
        }
    }

    /// Process a `handle_command` outcome (command-mode `processStepResult`): apply the action,
    /// setting up forwarding when the result is an *AndRepeat* variant.
    fn dispatch(&mut self, entry: StepEntry, cmd: Action, mut outcome: StepOutcome) {
        self.apply_effects(&mut outcome);
        match outcome.action {
            StepAction::Continue | StepAction::Repeat => {
                // Same step keeps waiting (multi-command step) — re-arm its prompt.
                self.pending_prompt = outcome.prompt;
                self.current = Some(entry);
            }
            StepAction::NextStep => { self.current = None; }
            StepAction::GotoLabel => {
                let label = outcome.goto_label.expect("goto without label");
                self.stack.goto_label(&label).expect("goto label present");
                self.current = None;
            }
            StepAction::NextStepAndRepeat => { self.current = None; self.forwarded = Some(cmd); }
            StepAction::GotoLabelAndRepeat => {
                let label = outcome.goto_label.expect("goto without label");
                self.stack.goto_label(&label).expect("goto label present");
                self.current = None;
                self.forwarded = Some(cmd);
            }
        }
    }
}

// ── Sequence generators ───────────────────────────────────────────────────────────

/// Java `StartGame` generator (BB2025) — head through the coin/receive decisions:
/// InitStartGame → Spectators → Weather → CoinChoice → ReceiveChoice → [PettyCash,
/// BuyInducements, Setup, Kickoff — Phase D]. See `10_sequences.md` StartGame.
pub fn start_game_sequence() -> Vec<StepEntry> {
    vec![
        StepEntry::new(Step::InitStartGame),
        StepEntry::new(Step::Spectators),
        StepEntry::new(Step::Weather),
        // Kickoff(withCoinChoice) — coin/receive then the opening kickoff. (PettyCash/
        // BuyInducements are 0-effect for equal-TV lineman and are omitted for now.)
        StepEntry::new(Step::CoinChoice),
        StepEntry::new(Step::ReceiveChoice),
        StepEntry::new(Step::InitKickoff),
        StepEntry::new(Step::Setup), // kicking team
        StepEntry::new(Step::Setup), // receiving team
        StepEntry::new(Step::Kickoff),
        StepEntry::new(Step::KickoffScatterRoll),
        StepEntry::new(Step::KickoffResultRoll),
        // TODO(next): ApplyKickoffResult, CatchScatterThrowIn(bounce), EndKickoff, InitSelecting.
    ]
}

// ── shared test fixtures (used by engine.rs and agent.rs tests) ──
#[cfg(test)]
pub(crate) fn test_team(side: &str, dedicated_fans: i32) -> ffb_model::model::team::Team {
    ffb_model::model::team::Team {
        id: format!("{side}_lineman"), name: format!("{side} Linemen"),
        race: "lineman".into(), roster_id: "lineman".into(), coach: format!("Coach_{side}"),
        rerolls: 3, apothecaries: 1, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0,
        bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0, assistant_coaches: 0,
        fan_factor: 0, dedicated_fans, team_value: 1_000_000, treasury: 0,
        special_rules: vec![], players: vec![],
    }
}

#[cfg(test)]
pub(crate) fn new_game(seed: u64) -> GameState {
    use ffb_model::enums::Rules;
    let game = Game::new(test_team("home", 5), test_team("away", 7), Rules::Bb2025);
    let mut gs = GameState::from_game(game, seed);
    gs.push_sequence(start_game_sequence());
    gs
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── stack mechanics (moved from framework.rs; now carry a real Step) ──
    #[test]
    fn stack_push_sequence_runs_first_authored_first() {
        let mut s = StepStack::new();
        s.push_sequence(vec![
            StepEntry::new(Step::InitStartGame),
            StepEntry::new(Step::Spectators),
            StepEntry::labelled(Step::Weather, "weather"),
        ]);
        assert_eq!(s.pop().unwrap().id(), StepId::InitStartGame);
        assert_eq!(s.pop().unwrap().id(), StepId::Spectators);
        assert_eq!(s.pop().unwrap().id(), StepId::Weather);
    }

    #[test]
    fn goto_label_discards_until_label_on_top() {
        let mut s = StepStack::new();
        s.push(StepEntry::labelled(Step::Weather, "weather"));
        s.push(StepEntry::new(Step::Spectators));
        s.push(StepEntry::new(Step::InitStartGame));
        s.goto_label("weather").unwrap();
        assert_eq!(s.peek().unwrap().id(), StepId::Weather);
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn goto_unknown_label_errors() {
        let mut s = StepStack::new();
        s.push(StepEntry::new(Step::Spectators));
        assert!(s.goto_label("nope").is_err());
    }

    #[test]
    fn publish_walks_top_to_bottom_until_consumed() {
        // No pregame step consumes a param, so a published param propagates to the bottom and is
        // dropped without panicking — proves the walk is wired. (Consumption asserted in Phase D
        // once a param-reading step lands.)
        let mut s = StepStack::new();
        s.push(StepEntry::new(Step::ReceiveChoice));
        s.push(StepEntry::new(Step::CoinChoice));
        s.publish(&StepParameter::EndTurn(true));
        assert_eq!(s.len(), 2, "non-consuming publish leaves the stack intact");
    }

    #[test]
    fn pregame_consumes_d3_d3_d6_d6_then_waits_at_coin_prompt() {
        let seed = 1u64;
        let mut refrng = GameRng::new(seed);
        let exp_fan_home = refrng.d3();
        let exp_fan_away = refrng.d3();
        let exp_w = Weather::for_roll(refrng.d6() + refrng.d6());

        let mut gs = new_game(seed);
        gs.run_until_prompt();

        // 4 dice consumed (fan d3 x2, weather d6 x2); the coin's d2 is NOT rolled until the
        // guess arrives — the engine is now waiting at the coin prompt.
        assert_eq!(gs.rng.call_count, 4);
        assert_eq!(gs.game.status, GameStatus::Active);
        assert_eq!(gs.game.team_home.fan_factor, 5 + exp_fan_home);
        assert_eq!(gs.game.team_away.fan_factor, 7 + exp_fan_away);
        assert_eq!(gs.game.weather, exp_w);
        assert!(matches!(gs.current_prompt(), Some(AgentPrompt::CoinChoice { is_home: true })));
    }

    #[test]
    fn kickoff_scatter_rolls_d8_dir_then_d6_dist_and_places_ball() {
        // Characterization (per SEED1_DICE_MAP): direction d8 FIRST, distance d6 SECOND, ball
        // placed at the on-pitch landing. Pin the order + mechanic against a reference RNG.
        let seed = 99u64;
        let mut refrng = GameRng::new(seed);
        let exp_dir_roll = refrng.d8();
        let exp_dist = refrng.d6();
        let exp_dir = Direction::for_roll(exp_dir_roll).unwrap();
        let start = FieldCoordinate::new(13, 7); // mid-pitch — scatter stays on-pitch
        let (ex, ey) = scatter_coordinate(start.x, start.y, exp_dir, exp_dist);

        let game = Game::new(test_team("home", 0), test_team("away", 0), ffb_model::enums::Rules::Bb2025);
        let mut gs = GameState::from_game(game, seed);
        gs.game.field_model.ball_coordinate = Some(start);
        gs.push_sequence(vec![StepEntry::new(Step::KickoffScatterRoll)]);
        gs.run_until_prompt();

        assert_eq!(gs.rng.call_count, 2, "exactly d8 then d6");
        assert!(matches!(gs.events.as_slice(),
            [GameEvent::KickoffScatter { start: s, direction, distance }]
            if *s == start && *direction == exp_dir_roll && *distance == exp_dist));
        // mid-pitch landing is on-pitch → ball moved there.
        assert_eq!(gs.game.field_model.ball_coordinate, Some(FieldCoordinate::new(ex, ey)));
    }

    #[test]
    fn kickoff_result_rolls_2d6_and_maps_table() {
        // 2d6 → BB2025 kickoff table. Pin the order + that the mapped result is published/emitted.
        let seed = 99u64;
        let mut refrng = GameRng::new(seed);
        let total = refrng.d6() + refrng.d6();
        let exp = kickoff_result_from_kind(kickoff_event_bb2025(total).unwrap());

        let game = Game::new(test_team("home", 0), test_team("away", 0), ffb_model::enums::Rules::Bb2025);
        let mut gs = GameState::from_game(game, seed);
        gs.push_sequence(vec![StepEntry::new(Step::KickoffResultRoll)]);
        gs.run_until_prompt();

        assert_eq!(gs.rng.call_count, 2, "exactly 2d6");
        assert!(matches!(gs.events.as_slice(),
            [GameEvent::KickoffResultEvent { result }] if *result == exp));
    }

    #[test]
    fn coin_then_receive_drives_to_idle_with_correct_offense_and_dice_order() {
        let seed = 1u64;
        // Reference: after fan d3,d3 + weather d6,d6, the next game die is the coin d2.
        let mut refrng = GameRng::new(seed);
        let (_h, _a) = (refrng.d3(), refrng.d3());
        let (_w1, _w2) = (refrng.d6(), refrng.d6());
        let coin_is_heads = refrng.bool();

        let mut gs = new_game(seed);
        gs.run_until_prompt();
        assert_eq!(gs.rng.call_count, 4, "no coin die before the guess");

        // Agent guesses heads=true. Coin flip happens now (5th die = d2).
        gs.apply_action(Action::CoinChoice { heads: true });
        assert_eq!(gs.rng.call_count, 5, "coin flip is the 5th game die (d2)");
        let home_won = true == coin_is_heads;
        assert_eq!(gs.game.home_playing, home_won, "winner becomes the chooser");
        // CoinThrow emitted; now waiting at the receive prompt for the winner's team.
        assert!(gs.events.iter().any(|e| matches!(e, GameEvent::CoinThrow { home_won: hw } if *hw == home_won)));
        let chooser_team = if home_won { gs.game.team_home.id.clone() } else { gs.game.team_away.id.clone() };
        assert!(matches!(gs.current_prompt(), Some(AgentPrompt::ReceiveChoice { team_id }) if *team_id == chooser_team));

        // Winner chooses to receive. home_first_offense follows; kicker (home_playing) is the
        // opposite. The engine then drives through the opening kickoff (InitKickoff, Setup×2,
        // Kickoff, scatter d8+d6, result 2d6) and idles after the result roll.
        gs.apply_action(Action::ReceiveChoice { receive: true });
        let home_receives = if home_won { true } else { false }; // chooser receives
        assert_eq!(gs.game.home_first_offense, home_receives);
        assert_eq!(gs.game.home_playing, !home_receives, "kicker kicks (set up first; two Setup flips net to kicker)");
        assert!(gs.events.iter().any(|e| matches!(e, GameEvent::ReceiveChoice { receive, .. } if *receive == home_receives)));
        // Receive (0 dice) + InitKickoff/Setup×2/Kickoff (0 dice) + scatter d8,d6 + result d6,d6.
        assert_eq!(gs.rng.call_count, 9, "coin d2 + scatter d8,d6 + result d6,d6 = 5+4");
        assert!(gs.events.iter().any(|e| matches!(e, GameEvent::KickoffScatter { .. })));
        assert!(gs.events.iter().any(|e| matches!(e, GameEvent::KickoffResultEvent { .. })));
        assert!(gs.current_prompt().is_none(), "stack drains after result roll (apply/bounce TODO)");
    }
}
