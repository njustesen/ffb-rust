/// Skill behaviour implementations — the live skill-dispatch mechanism used at runtime.
///
/// Each edition module holds one struct per skill; `register_into` wires its
/// `StepModifierTrait` impls (see `model::step_modifier`) into a `SkillRegistry`
/// (see `registry`), consulted at runtime via `dispatch::execute_step_hooks` /
/// `dispatch::handle_skill_command`.
pub mod bb2016;
pub mod bb2020;
pub mod bb2025;
pub mod common;
pub mod mixed;
pub mod step_hook;
pub mod registry;
pub mod dispatch;
