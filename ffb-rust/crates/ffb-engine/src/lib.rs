/// Step-stack engine — the 1:1 Java port. Java (`com.fumbbl.ffb`) is the sole ground truth;
/// the old monolithic `GameEngine` has been removed entirely. See docs/step_port/.
pub mod step;
/// Decision-maker boundary: `Agent::act(&GameState) -> Action` (parity `RandomAgent`).
pub mod agent;
pub mod action;
/// Pure `&Game` legality queries (eligible players/targets), consumed by selection/action
/// steps. Audited against Java as each consuming step is ported.
pub mod legal_actions;

/// Skill behaviour hooks — Rust analogue of Java's `SkillBehaviour` + `StepModifier` system.
pub mod skill_behaviour;

/// Step factory infrastructure — `StepIdFactory` (name↔id mapping), `StepActionFactory` (step instantiation).
pub mod factory;

/// Engine-level model types — `StepModifier` trait and related infrastructure.
pub mod model;

/// Partial translations of InjuryContext (ffb-common) and InjuryResult (ffb-server).
pub mod injury;
pub mod injury_result;

/// Port of `com.fumbbl.ffb.server.model.DropPlayerContext` and `SteadyFootingContext`.
pub mod drop_player_context;

/// 1:1 translation of com.fumbbl.ffb.server.DiceInterpreter.
pub mod dice_interpreter;

/// 1:1 translation of com.fumbbl.ffb.server.ActionStatus.
pub mod action_status;

/// 1:1 translation of com.fumbbl.ffb.server.GameStartMode.
pub mod game_start_mode;

/// 1:1 translation of com.fumbbl.ffb.server.IdGenerator.
pub mod id_generator;

/// 1:1 translation of com.fumbbl.ffb.server.ServerMode.
pub mod server_mode;

/// 1:1 translation of com.fumbbl.ffb.server.PrayerState.
pub mod prayer_state;

/// 1:1 translation of com.fumbbl.ffb.server.marking.*.
pub mod marking;

/// 1:1 translation of com.fumbbl.ffb.server.SessionMode.
pub mod session_mode;

/// 1:1 translation of com.fumbbl.ffb.server.ActiveEffects.
pub mod active_effects;

/// 1:1 translation of com.fumbbl.ffb.server.Talk.
pub mod talk;

/// 1:1 translation of com.fumbbl.ffb.server.ReplayState.
pub mod replay_state;

/// Utility helpers — partial translations of com.fumbbl.ffb.server.util.* and mechanic calculators.
pub mod util;

/// Port of `com.fumbbl.ffb.server.mechanic.*` — casualty/injury calc utilities and edition mechanics.
pub mod mechanic;

/// Port of `com.fumbbl.ffb.server.inducements.*` — prayer handlers, card handlers.
pub mod inducements;


/// Parity debug tracing, enabled by setting the FFB_TRACE env var.
/// Used to gate stderr diagnostics (dodge rolls, negatrait rolls, agent picks)
/// that align with the Java ParityRunner's -Dffb.diceTrace output.
pub fn parity_trace_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("FFB_TRACE").is_some())
}
