use crate::skill_behaviour::SkillBehaviour;

/// BB2020 Swoop skill behaviour. StepModifier for Swoop: allows player to choose direction of
/// scatter/throw-in on a failed pass. Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.SwoopBehaviour`.
pub struct SwoopBehaviour;

impl SwoopBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for SwoopBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for SwoopBehaviour {
    fn name(&self) -> &'static str { "SwoopBehaviour" }

    /// Java `StepModifier` for Swoop: on pass, if player has Swoop, may modify scatter direction.
    /// Returns false always.
    /// TODO(hook-infra): needs scatter state.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        // TODO(hook-infra): step-specific state access (StepState.xxx)
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_is_noop_returns_false() {
        // Without step infra the hook always returns false.
        let b = SwoopBehaviour::new();
        assert_eq!(b.name(), "SwoopBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = SwoopBehaviour::default();
        assert_eq!(b.name(), "SwoopBehaviour");
    }
}
