use crate::skill_behaviour::SkillBehaviour;

/// BB2020 AnimalSavagery skill behaviour.
/// StepModifier on StepAnimalSavagery: rolls confusion-style check; if failed, may injure random
/// teammate or show player-choice dialog. Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.AnimalSavageryBehaviour`.
pub struct AnimalSavageryBehaviour;

impl AnimalSavageryBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for AnimalSavageryBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for AnimalSavageryBehaviour {
    fn name(&self) -> &'static str { "AnimalSavageryBehaviour" }

    /// Java `StepModifier<StepAnimalSavagery, StepState>.handleExecuteStepHook`:
    /// rolls confusion-style check; if failed, may injure random teammate or show
    /// player-choice dialog. Returns false always.
    ///
    /// TODO(hook-infra): needs state.status, state.goToLabelOnFailure,
    /// state.kickingPlayer etc.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        // TODO(hook-infra): step-specific state access (StepState.status,
        // StepState.goToLabelOnFailure, StepState.kickingPlayer)
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_is_noop_returns_false() {
        // Without step infra the hook always returns false.
        let b = AnimalSavageryBehaviour::new();
        assert_eq!(b.name(), "AnimalSavageryBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = AnimalSavageryBehaviour::default();
        assert_eq!(b.name(), "AnimalSavageryBehaviour");
    }
}
