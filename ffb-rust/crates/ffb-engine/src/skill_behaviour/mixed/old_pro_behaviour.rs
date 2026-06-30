use crate::skill_behaviour::SkillBehaviour;

/// Old Pro: may attempt to avoid being Stunned once per game (multi-edition).
///
/// Injury-modifier-only behaviour: registers an `OldProModification` injury modifier.
/// No step hook is registered; `execute_step_hook` is not overridden.
///
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.mixed.OldProBehaviour`.
pub struct OldProBehaviour;

impl OldProBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for OldProBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for OldProBehaviour {
    fn name(&self) -> &'static str { "OldProBehaviour" }
    // Injury-modifier-only behaviour — no step hook. execute_step_hook returns false (default).
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = OldProBehaviour::new();
        assert_eq!(b.name(), "OldProBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = OldProBehaviour::default();
        assert_eq!(b.name(), "OldProBehaviour");
    }
}
