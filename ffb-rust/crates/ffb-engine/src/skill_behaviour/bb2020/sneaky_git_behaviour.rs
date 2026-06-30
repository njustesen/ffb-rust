use crate::skill_behaviour::SkillBehaviour;

/// BB2020 SneakyGit skill behaviour. StepModifier on StepFoul: if fouler has SneakyGit, rolls 3+
/// to avoid referee ejection even on doubles. Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.SneakyGitBehaviour`.
pub struct SneakyGitBehaviour;

impl SneakyGitBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for SneakyGitBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for SneakyGitBehaviour {
    fn name(&self) -> &'static str { "SneakyGitBehaviour" }

    /// Java `StepModifier<StepFoul, StepState>.handleExecuteStepHook`: if fouler has SneakyGit,
    /// rolls 3+ to avoid referee ejection even on doubles. Returns false always.
    /// TODO(hook-infra): needs step state for foul roll result and ejection flag.
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
        let b = SneakyGitBehaviour::new();
        assert_eq!(b.name(), "SneakyGitBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = SneakyGitBehaviour::default();
        assert_eq!(b.name(), "SneakyGitBehaviour");
    }
}
