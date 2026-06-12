//! Step engine: the `Step` enum (concrete steps + dispatch), the `StepStack`, and the
//! `GameState` driver loop — the Rust port of Java `GameState.executeStep`.
//! See `docs/step_port/00_framework.md` (driver) and `10_sequences.md` (sequences).
//!
//! Steps are dispatched via the `Step` enum (no `dyn`). A step's `start`/`handle_command`
//! return a `StepOutcome` (next action + events + sub-sequences to push); the driver applies
//! the pushes and processes the action. This keeps borrows simple: the driver owns the stack
//! and current step, and hands a step only `&mut Game` + `&mut GameRng`.

use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::events::GameEvent;
use ffb_model::enums::{GameStatus, Weather};

use super::framework::{StepAction, StepId};

// ── Concrete steps ──────────────────────────────────────────────────────────────
// One variant per ported step (BB2025 lineman set grows here per docs/step_port/20_steps).
// Each carries its persistent fields; for the pregame steps that is nothing yet.

#[derive(Debug, Clone)]
pub enum Step {
    /// Game-start bookkeeping: mark the game active. (Java `StepInitStartGame`.)
    InitStartGame,
    /// Roll fan factor for home then away. (Java `mixed/start/StepSpectators`.)
    Spectators,
    /// Roll initial weather (2d6). (Java `game/start/StepWeather`.)
    Weather,
}

impl Step {
    pub fn id(&self) -> StepId {
        match self {
            Step::InitStartGame => StepId::InitStartGame,
            Step::Spectators => StepId::Spectators,
            Step::Weather => StepId::Weather,
        }
    }

    /// The step's `start()` body (Java `AbstractStep.start`). Pregame steps do all their work
    /// here and advance with `NextStep`. Steps that wait for a command return `Continue` and
    /// implement `handle_command` (added as those steps are ported).
    fn start(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match self {
            Step::InitStartGame => {
                game.status = GameStatus::Active;
                StepOutcome::next()
            }
            // Java StepSpectators: rollFanFactor() d3 for home, then d3 for away;
            // fanFactor = dedicatedFans + roll. No GameEvent (not in the state hash).
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
        }
    }
}

// ── Step outcome / stack ─────────────────────────────────────────────────────────

/// What a step produced: how to advance, the events it emitted, and any sub-sequences to push.
/// (Java folds these into `StepResult` + `pushSequence`; we return them so the driver — sole
/// owner of the stack — applies the pushes, avoiding aliasing `&mut StepStack` into the step.)
pub struct StepOutcome {
    pub action: StepAction,
    pub goto_label: Option<String>,
    pub events: Vec<GameEvent>,
    /// Sequences to push (authored order; the stack reverses them on push).
    pub pushes: Vec<Vec<StepEntry>>,
}

impl StepOutcome {
    pub fn next() -> Self {
        StepOutcome { action: StepAction::NextStep, goto_label: None, events: Vec::new(), pushes: Vec::new() }
    }
    pub fn cont() -> Self {
        StepOutcome { action: StepAction::Continue, goto_label: None, events: Vec::new(), pushes: Vec::new() }
    }
    pub fn with_event(mut self, e: GameEvent) -> Self { self.events.push(e); self }
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
}

// ── Driver ──────────────────────────────────────────────────────────────────────

/// The game driver — owns the model, RNG, step stack and current step. Port of Java
/// `GameState` (the executeStep/processStepResult loop, flattened to an explicit loop per
/// `00_framework.md` §7). Command handling (`handle_command`) is added as command-driven
/// steps are ported; this slice exercises the `start`/`NextStep` chain (pregame).
pub struct GameState {
    pub game: Game,
    pub rng: GameRng,
    stack: StepStack,
    current: Option<StepEntry>,
    /// Events accumulated since the last drain (the parity log reads these).
    pub events: Vec<GameEvent>,
}

impl GameState {
    pub fn new(game: Game, seed: u64) -> Self {
        GameState { game, rng: GameRng::new(seed), stack: StepStack::new(), current: None, events: Vec::new() }
    }

    pub fn push_sequence(&mut self, seq: Vec<StepEntry>) { self.stack.push_sequence(seq); }

    /// Drive the stack until it idles — i.e. a step yields `Continue` (waiting for a command)
    /// or the stack empties. Mirrors `GameState.executeStep`'s start-mode chain. Command-mode
    /// (handle_command) lands with the first command-driven step.
    pub fn run_until_idle(&mut self) {
        loop {
            if self.current.is_none() {
                self.current = match self.stack.pop() { Some(s) => Some(s), None => return };
            }
            let entry = self.current.as_ref().unwrap();
            let outcome = entry.step.start(&mut self.game, &mut self.rng);
            self.events.extend(outcome.events);
            for seq in outcome.pushes { self.stack.push_sequence(seq); }
            match outcome.action {
                StepAction::Continue | StepAction::Repeat => {
                    // Continue: wait for a command (none in this slice). Repeat: re-run start
                    // is not used by pregame steps; treated as idle for now.
                    return;
                }
                StepAction::NextStep | StepAction::NextStepAndRepeat => {
                    self.current = None; // pop+start the next step
                }
                StepAction::GotoLabel | StepAction::GotoLabelAndRepeat => {
                    let label = outcome.goto_label.expect("goto without label");
                    self.stack.goto_label(&label).expect("goto label present");
                    self.current = None;
                }
            }
        }
    }
}

// ── Sequence generators ───────────────────────────────────────────────────────────

/// Java `StartGame` generator (BB2025): InitStartGame → Spectators → Weather → [PettyCash,
/// BuyInducements, Kickoff — added next slice]. See `10_sequences.md` StartGame.
pub fn start_game_sequence() -> Vec<StepEntry> {
    vec![
        StepEntry::new(Step::InitStartGame),
        StepEntry::new(Step::Spectators),
        StepEntry::new(Step::Weather),
    ]
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

    // ── pregame characterization: StartGame runs and consumes d3,d3,d6,d6 in order ──
    fn test_team(side: &str, dedicated_fans: i32) -> ffb_model::model::team::Team {
        ffb_model::model::team::Team {
            id: format!("{side}_lineman"), name: format!("{side} Linemen"),
            race: "lineman".into(), roster_id: "lineman".into(), coach: format!("Coach_{side}"),
            rerolls: 3, apothecaries: 1, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0,
            bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0, assistant_coaches: 0,
            fan_factor: 0, dedicated_fans, team_value: 1_000_000, treasury: 0,
            special_rules: vec![], players: vec![],
        }
    }

    #[test]
    fn start_game_pregame_consumes_d3_d3_d6_d6_in_order() {
        use ffb_model::enums::Rules;
        let seed = 1u64;
        // Reference dice: the exact draws StartGame must consume, in order.
        let mut refrng = GameRng::new(seed);
        let exp_fan_home = refrng.d3();
        let exp_fan_away = refrng.d3();
        let exp_w1 = refrng.d6();
        let exp_w2 = refrng.d6();
        let exp_weather = Weather::for_roll(exp_w1 + exp_w2);

        let game = Game::new(test_team("home", 5), test_team("away", 7), Rules::Bb2025);
        let mut gs = GameState::new(game, seed);
        gs.push_sequence(start_game_sequence());
        gs.run_until_idle();

        // Exactly 4 dice consumed, in order d3,d3,d6,d6 (parity-critical).
        assert_eq!(gs.rng.call_count, 4, "StartGame must consume exactly 4 dice (fan d3 x2, weather d6 x2)");
        assert_eq!(gs.game.status, GameStatus::Active);
        assert_eq!(gs.game.team_home.fan_factor, 5 + exp_fan_home);
        assert_eq!(gs.game.team_away.fan_factor, 7 + exp_fan_away);
        assert_eq!(gs.game.weather, exp_weather);
        // One WeatherChange event from StepWeather.
        assert!(matches!(gs.events.as_slice(), [GameEvent::WeatherChange { weather }] if *weather == exp_weather));
    }
}
