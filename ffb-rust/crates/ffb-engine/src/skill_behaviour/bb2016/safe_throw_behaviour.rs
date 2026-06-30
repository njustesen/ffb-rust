use crate::skill_behaviour::SkillBehaviour;

/// Safe Throw: registers StepSafeThrow via registerStep — no execute_step_hook modifier.
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2016.SafeThrowBehaviour`.
pub struct SafeThrowBehaviour;

impl SafeThrowBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for SafeThrowBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for SafeThrowBehaviour {
    fn name(&self) -> &'static str { "SafeThrowBehaviour" }

    /// Java: only registers StepSafeThrow via `registerStep`; no `handleExecuteStepHook`
    /// override exists. Nothing to do here.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = SafeThrowBehaviour::new();
        assert_eq!(b.name(), "SafeThrowBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = SafeThrowBehaviour::default();
        assert_eq!(b.name(), "SafeThrowBehaviour");
    }
}
