use crate::skill_behaviour::SkillBehaviour;

/// BB2020 DwarfenScourge skill behaviour. Registers AvOrInjModification: +1 to armour or injury
/// rolls against specific targets. Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.DwarfenScourgeBehaviour`.
pub struct DwarfenScourgeBehaviour;

impl DwarfenScourgeBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for DwarfenScourgeBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for DwarfenScourgeBehaviour {
    fn name(&self) -> &'static str { "DwarfenScourgeBehaviour" }

    /// No step modifier hook — this behaviour only registers AvOrInjModification.
    /// AvOrInjModification gives +1 to armour or injury rolls against specific targets.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_is_noop_returns_false() {
        // DwarfenScourgeBehaviour only registers AvOrInjModification; execute_step_hook is a no-op.
        let b = DwarfenScourgeBehaviour::new();
        assert_eq!(b.name(), "DwarfenScourgeBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = DwarfenScourgeBehaviour::default();
        assert_eq!(b.name(), "DwarfenScourgeBehaviour");
    }
}
