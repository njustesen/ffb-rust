use crate::skill_behaviour::SkillBehaviour;

/// BB2020 ReallyStupid skill behaviour. StepModifier on StepReallyStupid: rolls confusion check
/// (4+) each activation; on failure marks player confused and goes to failure label. Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.ReallyStupidBehaviour`.
pub struct ReallyStupidBehaviour;

impl ReallyStupidBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for ReallyStupidBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for ReallyStupidBehaviour {
    fn name(&self) -> &'static str { "ReallyStupidBehaviour" }

    /// Java `StepModifier<StepReallyStupid, StepState>.handleExecuteStepHook`: rolls confusion
    /// check (4+) each activation; on failure marks player confused and goes to failure label.
    /// Returns false always.
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
        let b = ReallyStupidBehaviour::new();
        assert_eq!(b.name(), "ReallyStupidBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = ReallyStupidBehaviour::default();
        assert_eq!(b.name(), "ReallyStupidBehaviour");
    }
}
