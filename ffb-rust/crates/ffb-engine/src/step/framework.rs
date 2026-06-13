//! Step framework primitives — direct port of Java `server/step`.
//! See `docs/step_port/00_framework.md`. These types are behaviour-frozen against Java;
//! the Rust representations (Vec stack, typed param enum, flattened driver loop) are the
//! agreed perf/idiom choices that must NOT change observable semantics.

/// What a step's result tells the driver to do next. 1:1 with Java `StepAction`
/// (`00_framework.md` §2). Flags: (trigger_next_step, forward_command, trigger_goto,
/// trigger_repeat).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepAction {
    /// Stay current; wait for the next external command.
    Continue,
    /// Pop + `start()` the next step.
    NextStep,
    /// Re-invoke `repeat()` on this step until it stops requesting repeat.
    Repeat,
    /// Pop the stack down to a label, then start that step.
    GotoLabel,
    /// Pop (no `start()`) and re-deliver the same command to the new step.
    NextStepAndRepeat,
    /// Pop-to-label, then re-deliver the same command to the labelled step.
    GotoLabelAndRepeat,
}

impl StepAction {
    pub const fn trigger_next_step(self) -> bool {
        matches!(self, Self::NextStep | Self::GotoLabel | Self::NextStepAndRepeat | Self::GotoLabelAndRepeat)
    }
    pub const fn forward_command(self) -> bool {
        matches!(self, Self::NextStepAndRepeat | Self::GotoLabelAndRepeat)
    }
    pub const fn trigger_goto(self) -> bool {
        matches!(self, Self::GotoLabel | Self::GotoLabelAndRepeat)
    }
    pub const fn trigger_repeat(self) -> bool {
        matches!(self, Self::Repeat)
    }
}

/// Step-internal control flow for `handle_command` (1:1 Java `StepCommandStatus`).
/// NOTE: the driver IGNORES this — only `StepResult::next_action` advances the game
/// (`00_framework.md` §3). It exists so a step's own `handle_command` can branch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepCommandStatus {
    UnhandledCommand,
    ExecuteStep,
    SkipStep,
}

/// One result per step. `reset()` clears reports/animation/sound but NOT `next_action`
/// (matches Java `StepResult.reset`, called by sync_game_model after the action runs).
#[derive(Debug, Clone)]
pub struct StepResult {
    pub next_action: StepAction,
    /// Goto target label when `next_action` triggers a goto.
    pub next_action_param: Option<String>,
    /// Events accumulated this step (the Rust analogue of Java's ReportList).
    pub events: Vec<ffb_model::events::GameEvent>,
    /// Whether to flush a model-sync (kept for fidelity; always true in headless parity).
    pub synchronize: bool,
}

impl Default for StepResult {
    fn default() -> Self {
        StepResult { next_action: StepAction::Continue, next_action_param: None, events: Vec::new(), synchronize: true }
    }
}

impl StepResult {
    /// Flush point: clears reports (events), keeps `next_action`. Java `StepResult.reset`.
    pub fn reset(&mut self) {
        self.events.clear();
    }
}

/// Identifier per step kind. Subset = the BB2025 skill-less lineman set (see
/// `docs/step_port/20_steps/`); extended in Phase E. Mirrors Java `StepId`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StepId {
    // start / kickoff / setup
    InitStartGame, Spectators, Weather, Kickoff, Setup, KickoffScatterRoll,
    KickoffResultRoll, ApplyKickoffResult, EndKickoff, CoinChoice, ReceiveChoice, Touchback,
    // select / activation
    InitSelecting, EndSelecting, InitActivation, StandUp, JumpUp, ResetFumblerooskie,
    // move
    InitMoving, Move, GoForIt, MoveDodge, FallDown, EndMoving,
    // block
    InitBlocking, BlockRoll, BlockChoice, BlockDodge, Pushback, Followup, BothDown,
    EndBlocking, DropFallingPlayers, PlaceBall,
    // foul
    InitFouling, Foul, Referee, Bribes, EjectPlayer, EndFouling,
    // pass / hand-over / ball
    InitPassing, Pass, DispatchPassing, Intercept, ResolvePass, HandOver, MissedPass,
    EndPassing, PickUp, CatchScatterThrowIn,
    // common / end
    Apothecary, EndPlayerAction, EndTurn, EndGame, Mvp,
    // common / misc
    NoOp,
    // control
    GotoLabel, NextStep,
}

/// Typed step parameter (Java `StepParameter`/`StepParameterKey`). The variant is the key;
/// `equals-by-key` in Java maps to matching on the variant. Lineman subset; extended later.
#[derive(Debug, Clone)]
pub enum StepParameter {
    MoveStack(Vec<ffb_model::types::FieldCoordinate>),
    MoveStart(ffb_model::types::FieldCoordinate),
    CoordinateFrom(ffb_model::types::FieldCoordinate),
    CoordinateTo(ffb_model::types::FieldCoordinate),
    BlockDefenderId(String),
    FoulDefenderId(String),
    NrOfDice(i32),
    DiceIndex(usize),
    TargetCoordinate(ffb_model::types::FieldCoordinate),
    EndTurn(bool),
    EndPlayerAction(bool),
    GotoLabel(String),
    GotoLabelOnEnd(String),
    GotoLabelOnFailure(String),
    GotoLabelOnSuccess(String),
    UsingChainsaw(bool),
    /// The rolled kickoff event (StepKickoffResultRoll → StepApplyKickoffResult).
    KickoffResult(ffb_model::enums::KickoffResult),
    // … grow per 20_steps entries as steps are ported.
}

/// A published parameter carries the consume flag used while walking the stack top→bottom.
pub struct Published {
    pub param: StepParameter,
    pub consumed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_action_flags_match_java_table() {
        assert!(StepAction::NextStep.trigger_next_step() && !StepAction::NextStep.forward_command());
        assert!(StepAction::GotoLabel.trigger_goto() && StepAction::GotoLabel.trigger_next_step());
        assert!(StepAction::Repeat.trigger_repeat() && !StepAction::Repeat.trigger_next_step());
        assert!(StepAction::NextStepAndRepeat.forward_command());
        assert!(StepAction::GotoLabelAndRepeat.forward_command() && StepAction::GotoLabelAndRepeat.trigger_goto());
        assert!(!StepAction::Continue.trigger_next_step() && !StepAction::Continue.trigger_repeat());
    }
}
