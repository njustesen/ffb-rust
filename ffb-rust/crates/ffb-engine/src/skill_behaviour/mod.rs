/// Skill behaviour implementations — the Rust analogue of Java's
/// `com.fumbbl.ffb.server.model.SkillBehaviour` registration system.
///
/// Each struct represents one BB2025 skill and will register StepModifier hooks
/// once the step-hook infrastructure is ported.
///
/// Java interface: `ISkillBehaviour` (ffb-common, 3 methods).
/// Java base class: `SkillBehaviour<T>` (ffb-server, registers step/player modifiers).
pub mod bb2016;
pub mod bb2020;
pub mod bb2025;
pub mod common;
pub mod mixed;

/// Marker trait — the Rust analogue of Java's `ISkillBehaviour`.
/// `execute_step_hook` mirrors Java's `StepModifier.handleExecuteStepHook`.
/// TODO: add `get_player_modifiers()` and `has_injury_modifier()` when
///       `PlayerModifier` and `InjuryType` types are fully ported.
pub trait SkillBehaviour: Send + Sync {
    fn name(&self) -> &'static str;

    /// Java: `StepModifier.handleExecuteStepHook(IStep, StepState)` — called by steps at hook
    /// points. Returns `true` if the step should be consumed (stop further hook processing).
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}
