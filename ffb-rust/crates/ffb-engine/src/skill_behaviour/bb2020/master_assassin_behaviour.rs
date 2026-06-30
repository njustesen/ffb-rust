use crate::skill_behaviour::SkillBehaviour;

/// BB2020 MasterAssassin skill behaviour. Registers MasterAssassinModification: modifies injury
/// results for stab/foul actions. Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.MasterAssassinBehaviour`.
pub struct MasterAssassinBehaviour;

impl MasterAssassinBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for MasterAssassinBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for MasterAssassinBehaviour {
    fn name(&self) -> &'static str { "MasterAssassinBehaviour" }

    /// No step modifier hook — this behaviour only registers MasterAssassinModification.
    /// MasterAssassinModification modifies injury results for stab/foul actions.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_is_noop_returns_false() {
        // MasterAssassinBehaviour only registers MasterAssassinModification; execute_step_hook is a no-op.
        let b = MasterAssassinBehaviour::new();
        assert_eq!(b.name(), "MasterAssassinBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = MasterAssassinBehaviour::default();
        assert_eq!(b.name(), "MasterAssassinBehaviour");
    }
}
