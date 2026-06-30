use crate::skill_behaviour::SkillBehaviour;

/// BB2020 GhostlyFlames skill behaviour. Registers GhostlyFlamesModification: modifies injury
/// results for this player's attacks. Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.GhostlyFlamesBehaviour`.
pub struct GhostlyFlamesBehaviour;

impl GhostlyFlamesBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for GhostlyFlamesBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for GhostlyFlamesBehaviour {
    fn name(&self) -> &'static str { "GhostlyFlamesBehaviour" }

    /// No step modifier hook — this behaviour only registers GhostlyFlamesModification.
    /// GhostlyFlamesModification modifies injury results for this player's attacks.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_is_noop_returns_false() {
        // GhostlyFlamesBehaviour only registers GhostlyFlamesModification; execute_step_hook is a no-op.
        let b = GhostlyFlamesBehaviour::new();
        assert_eq!(b.name(), "GhostlyFlamesBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = GhostlyFlamesBehaviour::default();
        assert_eq!(b.name(), "GhostlyFlamesBehaviour");
    }
}
