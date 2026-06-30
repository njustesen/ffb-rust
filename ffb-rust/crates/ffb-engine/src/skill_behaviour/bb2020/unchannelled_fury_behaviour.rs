use crate::skill_behaviour::SkillBehaviour;

/// BB2020 UnchannelledFury skill behaviour. StepModifier on StepUnchannelledFury: rolls confusion
/// check (4+) each activation; on failure marks player and goes to failure label. Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.UnchannelledFuryBehaviour`.
pub struct UnchannelledFuryBehaviour;

impl UnchannelledFuryBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for UnchannelledFuryBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for UnchannelledFuryBehaviour {
    fn name(&self) -> &'static str { "UnchannelledFuryBehaviour" }

    /// Java `StepModifier<StepUnchannelledFury, StepState>.handleExecuteStepHook`: rolls confusion
    /// check (4+) each activation; on failure marks player and goes to failure label. Returns false
    /// always.
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
        let b = UnchannelledFuryBehaviour::new();
        assert_eq!(b.name(), "UnchannelledFuryBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = UnchannelledFuryBehaviour::default();
        assert_eq!(b.name(), "UnchannelledFuryBehaviour");
    }
}
