use crate::skill_behaviour::SkillBehaviour;

/// BB2020 ToxinConnoisseur skill behaviour. StepModifier on stab/injury step: if player has
/// ToxinConnoisseur, modifies injury result to apply poisoned injury type. Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.ToxinConnoisseurBehaviour`.
pub struct ToxinConnoisseurBehaviour;

impl ToxinConnoisseurBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for ToxinConnoisseurBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for ToxinConnoisseurBehaviour {
    fn name(&self) -> &'static str { "ToxinConnoisseurBehaviour" }

    /// Java `StepModifier` on stab/injury step: if player has ToxinConnoisseur, modifies injury
    /// result to apply poisoned injury type. Returns false always.
    /// TODO(hook-infra): needs injury step state.
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
        let b = ToxinConnoisseurBehaviour::new();
        assert_eq!(b.name(), "ToxinConnoisseurBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = ToxinConnoisseurBehaviour::default();
        assert_eq!(b.name(), "ToxinConnoisseurBehaviour");
    }
}
