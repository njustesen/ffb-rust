use crate::skill_behaviour::SkillBehaviour;

/// Crushing Blow: +1 to armour rolls after a Knock Down result (multi-edition).
///
/// Injury-modifier-only behaviour: registers a `CrushingBlowModification` injury modifier.
/// No step hook is registered; `execute_step_hook` is not overridden.
///
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.mixed.CrushingBlowBehaviour`.
pub struct CrushingBlowBehaviour;

impl CrushingBlowBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for CrushingBlowBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for CrushingBlowBehaviour {
    fn name(&self) -> &'static str { "CrushingBlowBehaviour" }
    // Injury-modifier-only behaviour — no step hook. execute_step_hook returns false (default).
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = CrushingBlowBehaviour::new();
        assert_eq!(b.name(), "CrushingBlowBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = CrushingBlowBehaviour::default();
        assert_eq!(b.name(), "CrushingBlowBehaviour");
    }
}
