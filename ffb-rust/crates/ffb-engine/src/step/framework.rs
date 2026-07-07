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
    // throw-team-mate (BB2020)
    AlwaysHungry, DispatchScatterPlayer, EndScatterPlayer, EndThrowTeamMate,
    InitScatterPlayer, InitThrowTeamMate, RightStuff, ThrowTeamMate, FumbleTtmPass,
    // kickoff / start (BB2020)
    InitKickoff, KickoffScatterRollAskAfter, KickoffAnimation, KickoffReturn,
    BuyCardsAndInducements, BuyInducements, BuyCards, PettyCash,
    // skills — movement / activation
    BlitzTurn, Jump, Swarming, HypnoticGaze, Shadowing,
    // skills — block
    BlockChainsaw, BreatheFire, Chomp, HitAndRun, Trickster, Juggernaut,
    BlockBallAndChain, MoveBallAndChain, DivingTackle, Dauntless, DauntlessMultiple,
    DoubleStrength, SetDefender, Stab, HandleDropPlayerContext,
    // skills — foul
    FoulChainsaw, FoulAppearance, FoulAppearanceMultiple,
    // skills — special
    AnimalSavagery, Animosity, BoneHead, ReallyStupid, WildAnimal, TakeRoot,
    ForgoneStalling, CheckStalling, StallingPlayer,
    AutoGazeZoat, BalefulHex, BlackInk, Bombardier, Horns, GettingEven,
    ProjectileVomit, QuickBite, RaidingParty, SafeThrow, Swoop, Tentacles,
    Treacherous, UnchannelledFury, WisdomOfTheWhiteDwarf, Wrestle,
    // pass / ball special
    HailMaryPass, PassBlock, DumpOff, DispatchDumpOff,
    // inducements / kickoff events
    InitInducement, EndInducement, WeatherMage, Wizard, ThrowARock,
    InitKickTeamMate, EndKickTeamMate, KickTeamMate, KickTeamMateDoubleRolled,
    // end of game / scoring
    AssignTouchdowns, InitEndGame, Winnings, PlayerLoss, FanFactor, DedicatedFans,
    MasterChef, RiotousRookies, Prayer, Prayers, PrayerRoll,
    // punt
    InitPunt, PuntDirection, PuntDistance, EndPunt,
    // bomb / special actions
    InitBomb, EndBomb, ResolveBomb, Bombardier2,
    // gaze
    SelectGazeTarget, SelectGazeTargetEnd, LookIntoMyEyes, InitLookIntoMyEyes,
    // multiblock
    ApothecaryMultiple, BlockRollMultiple, MultipleBlockFork, ReportStabInjury,
    // feeding / furiousoutburst
    InitFeeding, EndFeeding, EatTeamMate, AllYouCanEat,
    InitFuriousOutburst, FirstMoveFuriousOutburst, SecondMoveFuriousOutburst,
    EndFuriousOutburst,
    // misc special
    SpecialEffect, ConsumeParameter, SetActingPlayerAndTeam, SetActingTeam,
    StateMultipleRolls, SteadyFooting, PickMeUp, PileDriver, CatchOfTheDay,
    RecheckExplodeSkill, DropActingPlayer, DropDivingTackler,
    ThenIStartedBlastin, EndThenIStartedBlastin,
    ThrowKeg, EndThrowKeg,
    BlockStatistics, SelectBlitzTarget, SelectBlitzTargetEnd,
    RemoveTargetSelectionState, ResetToMove,
    PenaltyShootout, TrapDoor,
    // blitz
    Pro, RevertEndTurn,
    // blood lust / cards
    BloodLust, PlayCard,
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
    /// Whether a touchback must fire (StepKickoffScatterRoll → StepApplyKickoffResult → StepCatchScatterThrowIn → StepTouchback).
    Touchback(bool),
    // ── TTM parameters ──────────────────────────────────────────────────────────
    /// The ID of the thrown/kicked player (TTM sequence).
    ThrownPlayerId(Option<String>),
    /// The PlayerState of the thrown/kicked player at throw time (TTM sequence).
    ThrownPlayerState(ffb_model::enums::PlayerState),
    /// Whether the thrown/kicked player is carrying the ball (TTM sequence).
    ThrownPlayerHasBall(bool),
    /// The landing coordinate of the thrown/kicked player (None = clear it) (TTM sequence).
    ThrownPlayerCoordinate(Option<ffb_model::types::FieldCoordinate>),
    /// Whether the throw produces scatter rather than a straight path (TTM).
    ThrowScatter(bool),
    /// Whether the player being scattered was originally kicked (TTM sequence).
    IsKickedPlayer(bool),
    /// Whether the thrown player should be dropped on landing (TTM RightStuff).
    DropThrownPlayer(bool),
    /// Result of a pass/throw roll (TTM, pass sequence).
    PassResultParam(ffb_model::enums::PassResult),
    /// Old defender state before the throw (StepInitThrowTeamMate → downstream).
    OldDefenderState(ffb_model::enums::PlayerState),
    /// Whether the thrown player should crash-land (BB2020 TTM sequence).
    CrashLanding(bool),
    /// Scatter/throw-in mode: "scatterBall" etc. (string form until enum is ported).
    CatchScatterThrowInMode(ffb_model::model::catch_scatter_throw_in_mode::CatchScatterThrowInMode),
    /// The ID of the kicked player (KTM sequence alias for ThrownPlayerId).
    KickedPlayerId(Option<String>),
    /// PlayerState of the kicked player at kick time.
    KickedPlayerState(ffb_model::enums::PlayerState),
    /// Field coordinate of the kicked player (KTM sequence).
    KickedPlayerCoordinate(ffb_model::types::FieldCoordinate),
    /// Whether the kicked player is carrying the ball (KTM sequence).
    KickedPlayerHasBall(bool),
    // ── foul parameters ──────────────────────────────────────────────────────
    FoulerHasBall(bool),
    ArgueTheCallSuccessful(bool),
    CheckForgo(bool),
    // ── inducement parameters ────────────────────────────────────────────────
    InducementPhase(ffb_model::enums::InducementPhase),
    EndInducementPhase(bool),
    InducementGoldHome(i32),
    InducementGoldAway(i32),
    // ── scoring / end parameters ─────────────────────────────────────────────
    Touchdowns(i32),
    TeamId(String),
    TouchdownPlayerId(Option<String>),
    HomeTeam(bool),
    TvHome(i32),
    TvAway(i32),
    NewHalf(bool),
    EndGame(bool),
    // ── block parameters ─────────────────────────────────────────────────────
    BlockResult(ffb_model::enums::BlockResult),
    BlockRoll(Vec<i32>),
    BlockTargets(Vec<String>),
    NrOfDice2(i32),
    NumDice(i32),
    AskForBlockKind(bool),
    FollowupChoice(bool),
    StartingPushbackSquare(Option<ffb_model::types::PushbackSquare>),
    DefenderPosition(ffb_model::types::FieldCoordinate),
    DefenderPushed(bool),
    AttackerAlreadyDown(bool),
    AttackerPoisoned(bool),
    DefenderPoisoned(bool),
    PublishDefender(bool),
    ResetForFailedBlock(bool),
    PushedOnBall(bool),
    PushSelect(bool),
    MultiBlockDefenderId(Option<String>),
    // ── injury / apothecary parameters ───────────────────────────────────────
    InjuryResult(Box<crate::injury::InjuryResult>),
    InjuryTypeName(String),
    ApothecaryMode(ffb_model::enums::ApothecaryMode),
    ActingTeam(bool),
    OldPlayerState(ffb_model::enums::PlayerState),
    DroppedBallCarrier(Option<String>),
    DropPlayerContext(Box<crate::drop_player_context::DropPlayerContext>),
    // ── multiblock parameters ─────────────────────────────────────────────────
    PlayerIdToRemove(String),
    PlayerIdDauntlessSuccess(String),
    DoubleTargetStrengthForPlayer(String),
    DoubleTargetStrength(bool),
    UsingStab(bool),
    PlayerOnBallId(String),
    /// Java: BLOCK_ROLL_ID — identifies which BlockRoll entry this evaluation sequence belongs to.
    BlockRollId(i32),
    /// Java: SUPPRESS_EXTRA_EFFECT_HANDLING — suppresses extra effect processing in StepBlockChoice.
    SuppressExtraEffectHandling(bool),
    /// Java: SHOW_NAME_IN_REPORT — show defender name in block report.
    ShowNameInReport(bool),
    // ── movement parameters ───────────────────────────────────────────────────
    AllowMoveAfterPass(bool),
    AllowSecondBlockAction(bool),
    BallAndChainGfi(bool),
    BallAndChainRrSetting(Option<String>),
    DodgeRoll(i32),
    DontDropFumble(bool),
    AttemptPickUp(bool),
    PickUpOptional(bool),
    Jumped(bool),
    KickingPlayerCoordinate(ffb_model::types::FieldCoordinate),
    RevertEndTurn(bool),
    // ── pass parameters ───────────────────────────────────────────────────────
    PassAccurate(bool),
    PassDeviates(bool),
    PassFumble(bool),
    PassingDistance(ffb_model::enums::PassingDistance),
    CatcherId(Option<String>),
    InterceptorId(Option<String>),
    HailMaryPassFlag(bool),
    FailedCatch(bool),
    FailedPickUp(bool),
    FailedDeflectionConversion(bool),
    CatchAccuratePass(bool),
    CatchAccuratePassEmptySquare(bool),
    CatchHandOff(bool),
    CatchMissedPass(bool),
    CatchScatter(bool),
    CatchThrowIn(bool),
    CatchKickoff(bool),
    CatchBomb(bool),
    CatchAccurateBomb(bool),
    CatchAccurateBombEmptySquare(bool),
    CatchPunt(bool),
    DeflectedBomb(bool),
    Deflected(bool),
    // ── kickoff parameters ────────────────────────────────────────────────────
    KickoffBounds(ffb_model::types::FieldCoordinateBounds),
    KickoffStartCoordinate(ffb_model::types::FieldCoordinate),
    HandleReceivingTeam(bool),
    // ── gaze parameters ───────────────────────────────────────────────────────
    GazeVictimId(Option<String>),
    // ── skill / re-roll parameters ────────────────────────────────────────────
    ReRollUsed(bool),
    UsingBreakTackle(bool),
    UsingBreatheFire(bool),
    UsingBullseye(bool),
    UsingChomp(bool),
    UsingDivingTackle(bool),
    UsingModifyingSkill(bool),
    UsingPilingOn(bool),
    UsingShadowing(Option<bool>),
    UsingSwoop(bool),
    UsingVomit(bool),
    DtRerollAsked(bool),
    Dauntless(bool),
    SuccessfulDauntless(bool),
    SuccessfulPro(bool),
    // ── card / inducement parameters ─────────────────────────────────────────
    CardId(Option<String>),
    PlayCard(bool),
    BombExploded(bool),
    BombOutOfBounds(bool),
    // ── misc parameters ───────────────────────────────────────────────────────
    AdminMode(bool),
    AlternateGotoLabel(String),
    UseAlternateLabel(bool),
    ArmBarPlayerId(Option<String>),
    ChoosingTeamId(Option<String>),
    Direction(ffb_model::enums::Direction),
    Increment(i32),
    IgnoreNullValue(bool),
    Retain(bool),
    InSelect(bool),
    PlayerEnteringSquare(String),
    PlayerWasPushed(bool),
    PlayerLoss(bool),
    RiotousRookies(bool),
    FeedingAllowed(bool),
    FeedOnPlayerChoice(bool),
    SpecialEffectKey(String),
    StateMultipleRolls(bool),
    SteadyFootingContext(Box<crate::drop_player_context::SteadyFootingContext>),
    BloodLustAction(Option<ffb_model::enums::PlayerAction>),
    /// Java: RESET_PLAYER_ACTION — when set, StepResetToMove clears the stack and pushes
    /// a Move sequence with the given action. The parameter is consumed by the step.
    ResetPlayerAction(ffb_model::enums::PlayerAction),
    ShadowerWasPreviousDefender(bool),
    GotoLabelOnBlitz(String),
    GotoLabelOnDodge(String),
    GotoLabelOnFallDown(String),
    GotoLabelOnHailMaryPass(String),
    GotoLabelOnHandOver(String),
    GotoLabelOnJuggernaut(String),
    GotoLabelOnMissedPass(String),
    GotoLabelOnPushback(String),
    GotoLabelOnSavedFumble(String),
    TargetPlayerId(Option<String>),
    PlayerId(String),
    ThrowInCoordinate(ffb_model::types::FieldCoordinate),
    PrayerRoll(i32),
    PrayersBoughtHome(i32),
    PrayersBoughtAway(i32),
    UpdatePersistence(bool),
    ParametersToConsume(Vec<std::mem::Discriminant<StepParameter>>),
    ConsumeParameter(bool),
    DispatchPlayerAction(Option<ffb_model::enums::PlayerAction>),
    RollForEffect(bool),
    KtmModifier(ffb_model::model::kick_team_mate_range::KickTeamMateRange),
    BallKnockedLoose(bool),
    IgnoreActedFlag(bool),
    /// Java: ORIGINAL_BOMBARDIER — PassState.originalBombardier (player ID of the bomb thrower).
    OriginalBombardier(Option<String>),
    // … grow per 20_steps entries as steps are ported.
}

/// A published parameter carries the consume flag used while walking the stack top→bottom.
pub struct Published {
    pub param: StepParameter,
    pub consumed: bool,
}

// ── Step trait (individual-file step pattern) ────────────────────────────────

/// Outcome returned by individual step implementations.
/// Mirrors `StepOutcome` in engine.rs but is self-contained for step unit tests.
#[derive(Debug)]
pub struct StepOutcome {
    pub action: StepAction,
    pub goto_label: Option<String>,
    pub published: Vec<StepParameter>,
    /// Sub-sequences to push onto the stack (authored order; driver reverses on push).
    pub pushes: Vec<Vec<SequenceStep>>,
    /// Events accumulated this step (mirrors Java's ReportList).
    pub events: Vec<ffb_model::events::GameEvent>,
    /// Prompt to display to the agent (set when action == Continue).
    pub prompt: Option<ffb_model::prompts::AgentPrompt>,
    /// When true, the driver must clear the entire step stack before processing
    /// `pushes`. Mirrors Java `GameState.getStepStack().clear()` called from within
    /// a step (e.g. `StepResetToMove`, `StepSelectGazeTarget`).
    pub clear_stack: bool,
}

impl StepOutcome {
    pub fn next() -> Self {
        StepOutcome { action: StepAction::NextStep, goto_label: None, published: Vec::new(), pushes: Vec::new(), events: Vec::new(), prompt: None, clear_stack: false }
    }
    pub fn cont() -> Self {
        StepOutcome { action: StepAction::Continue, goto_label: None, published: Vec::new(), pushes: Vec::new(), events: Vec::new(), prompt: None, clear_stack: false }
    }
    pub fn goto(label: &str) -> Self {
        StepOutcome { action: StepAction::GotoLabel, goto_label: Some(label.to_owned()), published: Vec::new(), pushes: Vec::new(), events: Vec::new(), prompt: None, clear_stack: false }
    }
    pub fn repeat() -> Self {
        StepOutcome { action: StepAction::Repeat, goto_label: None, published: Vec::new(), pushes: Vec::new(), events: Vec::new(), prompt: None, clear_stack: false }
    }
    /// Mark this outcome to clear the entire step stack before pushing sub-sequences.
    /// Java: `getGameState().getStepStack().clear()`.
    pub fn with_clear_stack(mut self) -> Self {
        self.clear_stack = true;
        self
    }
    pub fn publish(mut self, p: StepParameter) -> Self {
        self.published.push(p);
        self
    }
    pub fn push_seq(mut self, seq: Vec<SequenceStep>) -> Self {
        self.pushes.push(seq);
        self
    }
    pub fn with_event(mut self, e: ffb_model::events::GameEvent) -> Self {
        self.events.push(e);
        self
    }
    pub fn with_events(mut self, evs: Vec<ffb_model::events::GameEvent>) -> Self {
        self.events.extend(evs);
        self
    }
    pub fn with_prompt(mut self, p: ffb_model::prompts::AgentPrompt) -> Self {
        self.prompt = Some(p);
        self
    }
}

/// Trait for individual step implementations (BB2016/BB2020 per-file pattern).
/// Steps receive `&mut Game` and `&mut GameRng` only — the driver owns the stack.
pub trait Step {
    fn id(&self) -> StepId;
    fn start(&mut self, game: &mut ffb_model::model::game::Game, rng: &mut ffb_model::util::rng::GameRng) -> StepOutcome;
    fn handle_command(&mut self, action: &crate::action::Action, game: &mut ffb_model::model::game::Game, rng: &mut ffb_model::util::rng::GameRng) -> StepOutcome;
    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

/// Test helper: create a minimal team for unit tests.
#[cfg(test)]
pub fn test_team(side: &str, dedicated_fans: i32) -> ffb_model::model::team::Team {
    ffb_model::model::team::Team {
        id: side.into(),
        name: side.into(),
        race: "human".into(),
        roster_id: "human".into(),
        coach: "coach".into(),
        rerolls: 0,
        apothecaries: 0,
        bribes: 0,
        master_chefs: 0,
        prayers_to_nuffle: 0,
        bloodweiser_kegs: 0,
        riotous_rookies: 0,
        cheerleaders: 0,
        assistant_coaches: 0,
        fan_factor: 0,
        dedicated_fans,
        team_value: 0,
        treasury: 0,
        special_rules: Vec::new(),
        players: Vec::new(),
        vampire_lord: false,
        necromancer: false,
    }
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

    #[test]
    fn step_outcome_next_has_next_step_action() {
        let out = StepOutcome::next();
        assert!(matches!(out.action, StepAction::NextStep));
        assert!(out.published.is_empty());
    }

    #[test]
    fn step_outcome_cont_has_continue_action() {
        assert!(matches!(StepOutcome::cont().action, StepAction::Continue));
    }

    #[test]
    fn step_outcome_goto_stores_label() {
        let out = StepOutcome::goto("MY_LABEL");
        assert!(matches!(out.action, StepAction::GotoLabel));
        assert_eq!(out.goto_label.as_deref(), Some("MY_LABEL"));
    }

    #[test]
    fn step_outcome_publish_accumulates_params() {
        let out = StepOutcome::next()
            .publish(StepParameter::EndTurn(true))
            .publish(StepParameter::EndTurn(false));
        assert_eq!(out.published.len(), 2);
    }

    #[test]
    fn sequence_step_new_has_no_label_or_params() {
        let s = SequenceStep::new(StepId::Apothecary);
        assert_eq!(s.step_id, StepId::Apothecary);
        assert!(s.label.is_none());
        assert!(s.params.is_empty());
    }
}

// ── CatchScatterThrowInMode re-export ─────────────────────────────────────────
/// Re-exported from `ffb_model` for use by step implementations without an extra import.
pub use ffb_model::model::catch_scatter_throw_in_mode::CatchScatterThrowInMode;

// ── SequenceStep — one entry in a generator's step list ──────────────────────
/// One step entry in a generator's sequence (mirrors Java `IStep` container in `Sequence`).
/// Holds the step type id, an optional goto-label, and parameters to set on start.
#[derive(Debug, Clone)]
pub struct SequenceStep {
    pub step_id: StepId,
    pub label: Option<String>,
    pub params: Vec<StepParameter>,
}

impl SequenceStep {
    /// Create a step entry with no label and no parameters.
    pub fn new(step_id: StepId) -> Self {
        Self { step_id, label: None, params: Vec::new() }
    }
    /// Create a step entry with parameters but no label.
    pub fn with_params(step_id: StepId, params: Vec<StepParameter>) -> Self {
        Self { step_id, label: None, params }
    }
    /// Create a step entry with a label (goto-target) and parameters.
    pub fn labelled(step_id: StepId, label: impl Into<String>, params: Vec<StepParameter>) -> Self {
        Self { step_id, label: Some(label.into()), params }
    }
}

// ── DeferredCommand / DeferredCommandId ──────────────────────────────────────
/// Mirrors Java `com.fumbbl.ffb.server.step.IStep.DeferredCommand`.
/// Commands queued on a step to be executed when the step is revisited.
pub trait DeferredCommand: Send + Sync {
    fn id(&self) -> DeferredCommandId;
    fn execute(&self, game: &mut ffb_model::model::game::Game) -> Vec<StepParameter>;
}

/// Identifier for a DeferredCommand (mirrors Java `DeferredCommandId` enum).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeferredCommandId {
    AnimalSavageryCancelAction,
    AnimalSavageryControl,
    DropPlayer,
    DropPlayerFromBomb,
    HitPlayerTurnOver,
    HitPlayer,
    RightStuff,
    StandUp,
}
