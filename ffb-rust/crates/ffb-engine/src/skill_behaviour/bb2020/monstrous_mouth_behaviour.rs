use crate::skill_behaviour::SkillBehaviour;

/// BB2020 MonstrousMouth skill behaviour. StepModifier on StepCatchScatterThrowIn: if catcher has
/// MonstrousMouth, enables catch reroll (same as Catch skill). Returns true when consumed. Mirrors
/// Java `com.fumbbl.ffb.server.skillbehaviour.bb2020.MonstrousMouthBehaviour`.
pub struct MonstrousMouthBehaviour;

impl MonstrousMouthBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for MonstrousMouthBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for MonstrousMouthBehaviour {
    fn name(&self) -> &'static str { "MonstrousMouthBehaviour" }

    /// Java `StepModifier<StepCatchScatterThrowIn, StepState>.handleExecuteStepHook`: if catcher
    /// has MonstrousMouth, sets reRolledAction=CATCH and reRollSource, sets
    /// state.rerollCatch=true, returns true. Currently returns false as step state is unavailable.
    /// TODO(hook-infra): needs state.catcher, step.setReRolledAction, state.rerollCatch.
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
        let b = MonstrousMouthBehaviour::new();
        assert_eq!(b.name(), "MonstrousMouthBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = MonstrousMouthBehaviour::default();
        assert_eq!(b.name(), "MonstrousMouthBehaviour");
    }
}
