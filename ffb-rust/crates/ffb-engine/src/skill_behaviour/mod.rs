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
pub mod step_hook;
pub mod registry;
pub mod dispatch;

/// Marker trait — the Rust analogue of Java's `ISkillBehaviour`.
/// `execute_step_hook` mirrors Java's `StepModifier.handleExecuteStepHook`.
/// `apply_modifier` mirrors Java's `registerModifier` lambda — called during level-up.
pub trait SkillBehaviour: Send + Sync {
    fn name(&self) -> &'static str;

    /// Java: `StepModifier.handleExecuteStepHook(IStep, StepState)` — called by steps at hook
    /// points. Returns `true` if the step should be consumed (stop further hook processing).
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }

    /// Java: `registerModifier(player -> ...)` lambda — applies a stat mutation on level-up.
    /// `position` provides the position's base stats used as the cap anchor.
    /// Default no-op for behaviours that don't register a player modifier.
    fn apply_modifier(
        &self,
        _player: &mut ffb_model::model::player::Player,
        _position: &ffb_model::model::roster_position::RosterPosition,
    ) {}
}
