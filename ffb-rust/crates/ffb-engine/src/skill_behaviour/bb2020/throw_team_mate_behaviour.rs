use crate::skill_behaviour::SkillBehaviour;

/// BB2020 ThrowTeamMate skill behaviour. StepModifier on StepThrowTeamMate: rolls throw-team-mate
/// pass roll, evaluates distance, handles fumble with reroll dialog. Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.ThrowTeamMateBehaviour`.
pub struct ThrowTeamMateBehaviour;

impl ThrowTeamMateBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for ThrowTeamMateBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for ThrowTeamMateBehaviour {
    fn name(&self) -> &'static str { "ThrowTeamMateBehaviour" }

    /// Java `StepModifier<StepThrowTeamMate, StepState>.handleExecuteStepHook`: rolls
    /// throw-team-mate pass roll, evaluates distance, handles fumble with reroll dialog. Returns
    /// false always.
    /// TODO(hook-infra): needs state for pass roll, distance, fumble.
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
        let b = ThrowTeamMateBehaviour::new();
        assert_eq!(b.name(), "ThrowTeamMateBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = ThrowTeamMateBehaviour::default();
        assert_eq!(b.name(), "ThrowTeamMateBehaviour");
    }
}
