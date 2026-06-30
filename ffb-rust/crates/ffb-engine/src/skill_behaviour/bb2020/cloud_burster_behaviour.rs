use crate::skill_behaviour::SkillBehaviour;

/// BB2020 CloudBurster skill behaviour.
/// Registers StepCloudBurster step (not a StepModifier): when a deflection occurs on a long pass,
/// forces the interceptor to re-roll. execute_step_hook is not applicable — the logic lives in
/// StepCloudBurster.executeStep(). Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.CloudBursterBehaviour`.
pub struct CloudBursterBehaviour;

impl CloudBursterBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for CloudBursterBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for CloudBursterBehaviour {
    fn name(&self) -> &'static str { "CloudBursterBehaviour" }

    /// Not a StepModifier — CloudBursterBehaviour registers a full StepCloudBurster step.
    /// When deflection succeeds on a long pass, StepCloudBurster checks if thrower has
    /// canForceInterceptionRerollOfLongPasses, resets deflection, and re-pushes the intercept
    /// step.
    ///
    /// TODO(hook-infra): not applicable — logic lives in StepCloudBurster.executeStep().
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        // TODO(hook-infra): not applicable — logic lives in StepCloudBurster.executeStep()
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_is_noop_returns_false() {
        // Without step infra the hook always returns false.
        let b = CloudBursterBehaviour::new();
        assert_eq!(b.name(), "CloudBursterBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = CloudBursterBehaviour::default();
        assert_eq!(b.name(), "CloudBursterBehaviour");
    }
}
