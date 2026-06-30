use crate::skill_behaviour::SkillBehaviour;

/// Savage Mauling: extra injury die when causing a Crowd Surf (multi-edition).
///
/// Injury-modifier-only behaviour: registers a `SavageMaulingModification` injury modifier.
/// No step hook is registered; `execute_step_hook` is not overridden.
///
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.mixed.SavageMaulingBehaviour`.
pub struct SavageMaulingBehaviour;

impl SavageMaulingBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for SavageMaulingBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for SavageMaulingBehaviour {
    fn name(&self) -> &'static str { "SavageMaulingBehaviour" }
    // Injury-modifier-only behaviour — no step hook. execute_step_hook returns false (default).
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = SavageMaulingBehaviour::new();
        assert_eq!(b.name(), "SavageMaulingBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = SavageMaulingBehaviour::default();
        assert_eq!(b.name(), "SavageMaulingBehaviour");
    }
}
