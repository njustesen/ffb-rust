
/// Watch Out: nearby teammates gain a bonus on armour rolls (multi-edition).
///
/// Extends `AbstractDodgingBehaviour` with `priority = 2` and `requireUnusedSkill = true`.
/// Delegates entirely to the abstract parent's step logic; no additional override.
///
/// The real `StepModifierTrait` logic is `AbstractDodgingStepModifier`, registered
/// directly by `registry.rs::build_bb2020`/`build_bb2025` as
/// `AbstractDodgingBehaviour::register_into(&mut reg, SkillId::WatchOut, 2, true)` —
/// see `skill_behaviour/mixed/abstract_dodging_behaviour.rs`. This struct is an
/// intentionally inert marker; it adds no behaviour on top of that registration.
///
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.mixed.WatchOutBehaviour`.
pub struct WatchOutBehaviour;

impl WatchOutBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for WatchOutBehaviour {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

}
