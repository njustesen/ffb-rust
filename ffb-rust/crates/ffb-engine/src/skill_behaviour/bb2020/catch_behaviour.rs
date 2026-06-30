use crate::skill_behaviour::SkillBehaviour;

/// BB2020 Catch skill behaviour.
/// StepModifier on StepCatchScatterThrowIn: if catcher has Catch skill, enables catch reroll
/// (reRolledAction=CATCH, state.rerollCatch=true). Returns true when consumed. Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.CatchBehaviour`.
pub struct CatchBehaviour;

impl CatchBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for CatchBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for CatchBehaviour {
    fn name(&self) -> &'static str { "CatchBehaviour" }

    /// Java `StepModifier<StepCatchScatterThrowIn, StepState>.handleExecuteStepHook`:
    /// if catcher has Catch skill, sets reRolledAction=CATCH and reRollSource,
    /// sets state.rerollCatch=true, returns true. Currently returns false as step state is
    /// unavailable.
    ///
    /// TODO(hook-infra): needs state.catcher, step.setReRolledAction/setReRollSource,
    /// state.rerollCatch.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        // TODO(hook-infra): step-specific state access (state.catcher,
        // step.setReRolledAction, step.setReRollSource, state.rerollCatch)
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_is_noop_returns_false() {
        // Without step infra the hook always returns false.
        let b = CatchBehaviour::new();
        assert_eq!(b.name(), "CatchBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = CatchBehaviour::default();
        assert_eq!(b.name(), "CatchBehaviour");
    }
}
