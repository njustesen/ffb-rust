use crate::skill_behaviour::SkillBehaviour;

/// BB2020 BloodLust skill behaviour.
/// StepModifier on StepBloodLust: if WAIT_FOR_ACTION_CHANGE dispatches action; else rolls blood
/// lust check (2+ modified by good conditions), handles reroll and action change dialog. Mirrors
/// Java `com.fumbbl.ffb.server.skillbehaviour.bb2020.BloodLustBehaviour`.
pub struct BloodLustBehaviour;

impl BloodLustBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for BloodLustBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for BloodLustBehaviour {
    fn name(&self) -> &'static str { "BloodLustBehaviour" }

    /// Java `StepModifier<StepBloodLust, StepState>.handleExecuteStepHook`:
    /// if WAIT_FOR_ACTION_CHANGE dispatches action; else rolls blood lust check (2+ modified
    /// by good conditions), handles reroll and action change dialog. Returns false always.
    ///
    /// TODO(hook-infra): needs state.status, state.bloodlustAction,
    /// state.goToLabelOnFailure.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        // TODO(hook-infra): step-specific state access (StepState.status,
        // StepState.bloodlustAction, StepState.goToLabelOnFailure)
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_is_noop_returns_false() {
        // Without step infra the hook always returns false.
        let b = BloodLustBehaviour::new();
        assert_eq!(b.name(), "BloodLustBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = BloodLustBehaviour::default();
        assert_eq!(b.name(), "BloodLustBehaviour");
    }
}
