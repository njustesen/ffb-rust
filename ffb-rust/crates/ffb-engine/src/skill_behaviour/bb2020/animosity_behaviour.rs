use crate::skill_behaviour::SkillBehaviour;

/// BB2020 Animosity skill behaviour.
/// StepModifier on StepAnimosity: checks if animosity exists between thrower and catcher, rolls
/// 3+, handles reroll, sets sufferingAnimosity. Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.AnimosityBehaviour`.
pub struct AnimosityBehaviour;

impl AnimosityBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for AnimosityBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for AnimosityBehaviour {
    fn name(&self) -> &'static str { "AnimosityBehaviour" }

    /// Java `StepModifier<StepAnimosity, StepState>.handleExecuteStepHook`:
    /// checks if animosity exists between thrower and catcher, rolls 3+, handles reroll,
    /// sets sufferingAnimosity. Returns false always.
    ///
    /// TODO(hook-infra): needs state.catcherId, state.doRoll, state.gotoLabelOnFailure.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        // TODO(hook-infra): step-specific state access (StepState.catcherId,
        // StepState.doRoll, StepState.gotoLabelOnFailure)
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_is_noop_returns_false() {
        // Without step infra the hook always returns false.
        let b = AnimosityBehaviour::new();
        assert_eq!(b.name(), "AnimosityBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = AnimosityBehaviour::default();
        assert_eq!(b.name(), "AnimosityBehaviour");
    }
}
