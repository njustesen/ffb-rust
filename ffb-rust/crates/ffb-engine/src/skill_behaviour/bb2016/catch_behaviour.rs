use crate::skill_behaviour::SkillBehaviour;

/// Catch: +1 modifier on all catch rolls; enables catch skill reroll.
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2016.CatchBehaviour`.
pub struct CatchBehaviour;

impl CatchBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for CatchBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for CatchBehaviour {
    fn name(&self) -> &'static str { "CatchBehaviour" }

    /// Java `StepCatch.handleExecuteStepHook` logic (condensed):
    ///
    /// 1. Check that the active catcher has the Catch skill.
    /// 2. Set `step.reRolledAction = CATCH`.
    /// 3. Set `step.reRollSource  = skill.getRerollSource(CATCH)`.
    /// 4. Set `state.rerollCatch  = true`.
    /// 5. Return `true` — stops further hook processing for this step point.
    ///
    /// Note: the hook fires before the catch roll is made so that the step
    /// machinery can offer the skill reroll if the initial roll fails.
    ///
    /// TODO(hook-infra): step-specific state (StepState.rerollCatch,
    ///                   step.reRolledAction, step.reRollSource).
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = CatchBehaviour::new();
        assert_eq!(b.name(), "CatchBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = CatchBehaviour::default();
        assert_eq!(b.name(), "CatchBehaviour");
    }
}
