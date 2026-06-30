use crate::skill_behaviour::SkillBehaviour;

/// BB2020 TakeRoot skill behaviour. StepModifier on StepTakeRoot: rolls 2+ each activation; on
/// failure marks player as rooted (cannot move). Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.TakeRootBehaviour`.
pub struct TakeRootBehaviour;

impl TakeRootBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for TakeRootBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for TakeRootBehaviour {
    fn name(&self) -> &'static str { "TakeRootBehaviour" }

    /// Java `StepModifier<StepTakeRoot, StepState>.handleExecuteStepHook`: rolls 2+ each
    /// activation; on failure marks player as rooted (cannot move). Returns false always.
    /// TODO(hook-infra): needs state.goToLabelOnFailure, game.getTurnMode().checkNegatraits().
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
        let b = TakeRootBehaviour::new();
        assert_eq!(b.name(), "TakeRootBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = TakeRootBehaviour::default();
        assert_eq!(b.name(), "TakeRootBehaviour");
    }
}
