use crate::skill_behaviour::SkillBehaviour;

/// BB2020 PilingOn skill behaviour. StepModifier on StepDropFallingPlayers: after knockdown checks
/// if PilingOn player can re-roll injury, shows dialog, rolls 2+, handles Brawler reroll and
/// WeepingDagger interaction. Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.PilingOnBehaviour`.
pub struct PilingOnBehaviour;

impl PilingOnBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for PilingOnBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for PilingOnBehaviour {
    fn name(&self) -> &'static str { "PilingOnBehaviour" }

    /// Java `StepModifier<StepDropFallingPlayers, StepState>.handleExecuteStepHook`: after
    /// knockdown checks if PilingOn player can re-roll injury, shows dialog (DialogPilingOn),
    /// rolls 2+, handles Brawler reroll and WeepingDagger interaction. Returns false always.
    /// TODO(hook-infra): needs state.usingPilingOn, state.foulerId, state.victim, game option
    /// checks.
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
        let b = PilingOnBehaviour::new();
        assert_eq!(b.name(), "PilingOnBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = PilingOnBehaviour::default();
        assert_eq!(b.name(), "PilingOnBehaviour");
    }
}
