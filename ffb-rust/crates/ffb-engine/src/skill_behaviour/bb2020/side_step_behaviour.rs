use crate::skill_behaviour::SkillBehaviour;

/// BB2020 SideStep skill behaviour. StepModifier on StepPushback: if defender has SideStep,
/// changes pushback mode to allow player to choose destination square. Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.SideStepBehaviour`.
pub struct SideStepBehaviour;

impl SideStepBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for SideStepBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for SideStepBehaviour {
    fn name(&self) -> &'static str { "SideStepBehaviour" }

    /// Java `StepModifier<StepPushback, StepState>.handleExecuteStepHook`: if defender has
    /// SideStep, changes pushback mode to allow player to choose destination square. Returns false
    /// always.
    /// TODO(hook-infra): needs state.defender, state.pushbackSquares,
    /// state.startingPushbackSquare.
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
        let b = SideStepBehaviour::new();
        assert_eq!(b.name(), "SideStepBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = SideStepBehaviour::default();
        assert_eq!(b.name(), "SideStepBehaviour");
    }
}
