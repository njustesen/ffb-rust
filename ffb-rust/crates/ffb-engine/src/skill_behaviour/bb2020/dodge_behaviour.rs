
/// BB2020 Dodge skill behaviour. Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.DodgeBehaviour`, which just calls
/// `super(1, false)` on `AbstractDodgingBehaviour` with no BB2020-specific override.
///
/// The real `StepModifierTrait` logic (dodge-choice default, `ReportSkillUse`) is
/// `AbstractDodgingStepModifier`, registered directly by
/// `registry.rs::build_bb2020` as `AbstractDodgingBehaviour::register_into(&mut reg,
/// SkillId::Dodge, 1, false)` — see `skill_behaviour/mixed/abstract_dodging_behaviour.rs`.
/// This type is an intentionally inert marker (matches the BB2016 `DodgeBehaviour`
/// precedent of not double-registering already-real logic).
pub struct DodgeBehaviour;

impl DodgeBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for DodgeBehaviour {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

}
