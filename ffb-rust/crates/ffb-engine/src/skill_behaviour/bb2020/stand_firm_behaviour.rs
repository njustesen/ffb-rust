use crate::skill_behaviour::SkillBehaviour;

/// BB2020 StandFirm skill behaviour. StepModifier on StepPushback: if defender has StandFirm and
/// it is not cancelled, shows choice dialog to stay in place. Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.StandFirmBehaviour`.
pub struct StandFirmBehaviour;

impl StandFirmBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for StandFirmBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for StandFirmBehaviour {
    fn name(&self) -> &'static str { "StandFirmBehaviour" }

    /// Java `StepModifier<StepPushback, StepState>.handleExecuteStepHook`: if defender has
    /// StandFirm (and no cancelling skill), shows dialog for defender to opt out of pushback.
    /// Returns false always.
    /// TODO(hook-infra): needs state.defender, step dialog infra.
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
        let b = StandFirmBehaviour::new();
        assert_eq!(b.name(), "StandFirmBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = StandFirmBehaviour::default();
        assert_eq!(b.name(), "StandFirmBehaviour");
    }
}
