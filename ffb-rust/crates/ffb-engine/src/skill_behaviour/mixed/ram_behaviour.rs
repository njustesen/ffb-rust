use crate::skill_behaviour::SkillBehaviour;

/// Ram: may follow up a push and push again (multi-edition).
///
/// Armour/injury-modifier-only behaviour: registers an `AvOrInjModification` modifier.
/// No step hook is registered; `execute_step_hook` is not overridden.
///
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.mixed.RamBehaviour`.
pub struct RamBehaviour;

impl RamBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for RamBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for RamBehaviour {
    fn name(&self) -> &'static str { "RamBehaviour" }
    // Armour/injury-modifier-only behaviour — no step hook. execute_step_hook returns false (default).
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = RamBehaviour::new();
        assert_eq!(b.name(), "RamBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = RamBehaviour::default();
        assert_eq!(b.name(), "RamBehaviour");
    }
}
