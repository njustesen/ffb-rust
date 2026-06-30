use crate::skill_behaviour::SkillBehaviour;

/// BB2020 Dodge skill behaviour.
/// Extends AbstractDodgingBehaviour with +1 dodge modifier. execute_step_hook delegates to
/// AbstractDodgingBehaviour step modifier logic. Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.DodgeBehaviour`.
pub struct DodgeBehaviour;

impl DodgeBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for DodgeBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for DodgeBehaviour {
    fn name(&self) -> &'static str { "DodgeBehaviour" }

    /// Extends AbstractDodgingBehaviour with +1 dodge modifier (BB2020).
    /// execute_step_hook delegates to AbstractDodgingBehaviour step modifier logic.
    ///
    /// TODO(hook-infra): actual modifier application happens in StepModifier registered by
    /// AbstractDodgingBehaviour.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        // TODO(hook-infra): step-specific state access (StepState.xxx) not yet
        // available — implement fully once the step-hook infrastructure is ported.
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_is_noop_returns_false() {
        // Without step infra the hook always returns false.
        let b = DodgeBehaviour::new();
        assert_eq!(b.name(), "DodgeBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = DodgeBehaviour::default();
        assert_eq!(b.name(), "DodgeBehaviour");
    }
}
