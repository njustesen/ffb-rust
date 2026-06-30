use crate::skill_behaviour::SkillBehaviour;

/// Blind Rage: player must block or blitz each turn, may ignore Bone Head (multi-edition).
///
/// Only `handleCommandHook` is active in Java (sets `reRolledAction = DAUNTLESS` and
/// `reRollSource = BLIND_RAGE`, or clears them). The `execute_step_hook` is not overridden
/// in the Java source — no step logic runs here.
///
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.mixed.BlindRageBehaviour`.
pub struct BlindRageBehaviour;

impl BlindRageBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for BlindRageBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for BlindRageBehaviour {
    fn name(&self) -> &'static str { "BlindRageBehaviour" }
    // Registers on StepDauntless. Only handleCommandHook is active in Java —
    // no execute_step_hook override. Returns false (default).
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = BlindRageBehaviour::new();
        assert_eq!(b.name(), "BlindRageBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = BlindRageBehaviour::default();
        assert_eq!(b.name(), "BlindRageBehaviour");
    }
}
