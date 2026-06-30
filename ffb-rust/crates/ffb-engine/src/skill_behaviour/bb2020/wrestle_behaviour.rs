use crate::skill_behaviour::SkillBehaviour;

/// BB2020 Wrestle skill behaviour. StepModifier on StepBlock: if attacker has Wrestle and block
/// would make attacker prone, may use Wrestle to make both players prone instead. Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.WrestleBehaviour`.
pub struct WrestleBehaviour;

impl WrestleBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for WrestleBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for WrestleBehaviour {
    fn name(&self) -> &'static str { "WrestleBehaviour" }

    /// Java `StepModifier<StepBlock, StepState>.handleExecuteStepHook`: if attacker has Wrestle
    /// and block would make attacker prone, may use Wrestle to make both players prone instead.
    /// Returns false always.
    /// TODO(hook-infra): needs block step state, both-down result check.
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
        let b = WrestleBehaviour::new();
        assert_eq!(b.name(), "WrestleBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = WrestleBehaviour::default();
        assert_eq!(b.name(), "WrestleBehaviour");
    }
}
