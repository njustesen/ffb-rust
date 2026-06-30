use crate::skill_behaviour::SkillBehaviour;

/// BB2020 Slayer skill behaviour. StepModifier on injury step: if attacker has Slayer and target
/// is a Big Guy, applies additional injury modifier. Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.SlayerBehaviour`.
pub struct SlayerBehaviour;

impl SlayerBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for SlayerBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for SlayerBehaviour {
    fn name(&self) -> &'static str { "SlayerBehaviour" }

    /// Java `StepModifier` on injury step: if attacker has Slayer and target is a Big Guy, applies
    /// additional injury modifier. Returns false always.
    /// TODO(hook-infra): needs injury step state, target player type check.
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
        let b = SlayerBehaviour::new();
        assert_eq!(b.name(), "SlayerBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = SlayerBehaviour::default();
        assert_eq!(b.name(), "SlayerBehaviour");
    }
}
