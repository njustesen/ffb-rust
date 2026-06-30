/// Port of `com.fumbbl.ffb.server.model.DropPlayerContext` and
/// `com.fumbbl.ffb.server.model.SteadyFootingContext`.
use ffb_model::enums::ApothecaryMode;
use crate::injury::InjuryResult;

// ── VictimStateKey ────────────────────────────────────────────────────────────

/// Maps the Java `StepParameterKey` used as `victimStateKey` in `DropPlayerContext`.
/// Only the variants that carry a `PlayerState` value are listed here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VictimStateKey {
    OldDefenderState,
    ThrownPlayerState,
    OldPlayerState,
    KickedPlayerState,
}

// ── DropPlayerContext ─────────────────────────────────────────────────────────

/// Port of `com.fumbbl.ffb.server.model.DropPlayerContext`.
///
/// Carries all information needed to decide how a player should be dropped after
/// an injury or pushback (requires armour break, end-turn trigger, apothecary mode, etc.).
/// Consumed by `StepHandleDropPlayerContext`.
#[derive(Debug, Clone)]
pub struct DropPlayerContext {
    /// Java: injuryResult
    pub injury_result: Option<Box<InjuryResult>>,
    /// Java: endTurn
    pub end_turn: bool,
    /// Java: eligibleForSafePairOfHands
    pub eligible_for_safe_pair_of_hands: bool,
    /// Java: requiresArmourBreak
    pub requires_armour_break: bool,
    /// Java: alreadyDropped
    pub already_dropped: bool,
    /// Java: modifiedInjuryEndsTurn
    pub modified_injury_ends_turn: bool,
    /// Java: endTurnWithoutKnockdown
    pub end_turn_without_knockdown: bool,
    /// Java: label — goto-label hint for StepHandleDropPlayerContext
    pub label: Option<String>,
    /// Java: playerId
    pub player_id: Option<String>,
    /// Java: apothecaryMode
    pub apothecary_mode: Option<ApothecaryMode>,
    /// Java: victimStateKey
    pub victim_state_key: Option<VictimStateKey>,
    /// Java: additionalVictimStateKeys
    pub additional_victim_state_keys: Vec<VictimStateKey>,
}

impl Default for DropPlayerContext {
    fn default() -> Self { Self::new() }
}

impl DropPlayerContext {
    pub fn new() -> Self {
        Self {
            injury_result: None,
            end_turn: false,
            eligible_for_safe_pair_of_hands: false,
            requires_armour_break: false,
            already_dropped: false,
            modified_injury_ends_turn: false,
            end_turn_without_knockdown: false,
            label: None,
            player_id: None,
            apothecary_mode: None,
            victim_state_key: None,
            additional_victim_state_keys: Vec::new(),
        }
    }

    /// Mirrors the Java builder shorthand used by most callers:
    /// `DropPlayerContextBuilder.builder().injuryResult(...).playerId(...).apothecaryMode(...).eligibleForSafePairOfHands(true).build()`
    pub fn with_injury(
        injury_result: InjuryResult,
        player_id: String,
        apothecary_mode: ApothecaryMode,
        eligible_for_safe_pair_of_hands: bool,
    ) -> Self {
        Self {
            injury_result: Some(Box::new(injury_result)),
            player_id: Some(player_id),
            apothecary_mode: Some(apothecary_mode),
            eligible_for_safe_pair_of_hands,
            ..Self::new()
        }
    }
}

// ── SteadyFootingContext ──────────────────────────────────────────────────────

/// Port of `com.fumbbl.ffb.server.model.SteadyFootingContext`.
///
/// Holds one of three inner types (mirrors Java's three-constructor pattern):
/// - `DropPlayer` — most common; carries a full `DropPlayerContext`
/// - `InjuryResult` — when only the result (not the drop decision) is needed
/// - `InjuryTypeName` — when only the injury type is available (stored as class name string)
///
/// On a Steady Footing failure the context is forwarded to the next step via
/// `DROP_PLAYER_CONTEXT`, `INJURY_RESULT`, or `INJURY_TYPE` parameters respectively.
/// On success the context is discarded (the fall is cancelled).
#[derive(Debug, Clone)]
pub struct SteadyFootingContext {
    /// The wrapped payload (one of three variants).
    pub inner: SteadyFootingInner,
    // Java: List<DeferredCommand> deferredCommands — not yet ported; always empty.
}

/// Discriminated payload inside `SteadyFootingContext`.
#[derive(Debug, Clone)]
pub enum SteadyFootingInner {
    DropPlayer(Box<DropPlayerContext>),
    InjuryResult(Box<InjuryResult>),
    /// Java: InjuryTypeServer<?> — stored as class name string (mirrors StepParameter::InjuryTypeName).
    InjuryTypeName(String),
}

impl SteadyFootingContext {
    pub fn from_drop_player(ctx: DropPlayerContext) -> Self {
        Self { inner: SteadyFootingInner::DropPlayer(Box::new(ctx)) }
    }

    pub fn from_injury_result(result: InjuryResult) -> Self {
        Self { inner: SteadyFootingInner::InjuryResult(Box::new(result)) }
    }

    pub fn from_injury_type_name(name: String) -> Self {
        Self { inner: SteadyFootingInner::InjuryTypeName(name) }
    }

    /// Java: `getApothecaryMode()` — delegates to whichever inner holds the mode.
    pub fn get_apothecary_mode(&self) -> ApothecaryMode {
        match &self.inner {
            SteadyFootingInner::DropPlayer(ctx) => {
                ctx.apothecary_mode.unwrap_or(ApothecaryMode::Attacker)
            }
            SteadyFootingInner::InjuryResult(r) => {
                r.injury_context().get_apothecary_mode()
            }
            SteadyFootingInner::InjuryTypeName(_) => ApothecaryMode::Attacker,
        }
    }

    /// Java: derives `playerId` from whichever inner holds it.
    pub fn get_player_id(&self) -> Option<&str> {
        match &self.inner {
            SteadyFootingInner::DropPlayer(ctx) => ctx.player_id.as_deref(),
            SteadyFootingInner::InjuryResult(r) => r.injury_context().defender_id.as_deref(),
            SteadyFootingInner::InjuryTypeName(_) => None, // resolved from acting player at runtime
        }
    }

    pub fn drop_player_context(&self) -> Option<&DropPlayerContext> {
        match &self.inner {
            SteadyFootingInner::DropPlayer(ctx) => Some(ctx),
            _ => None,
        }
    }

    pub fn injury_result(&self) -> Option<&InjuryResult> {
        match &self.inner {
            SteadyFootingInner::InjuryResult(r) => Some(r),
            _ => None,
        }
    }

    pub fn injury_type_name(&self) -> Option<&str> {
        match &self.inner {
            SteadyFootingInner::InjuryTypeName(n) => Some(n),
            _ => None,
        }
    }
}
