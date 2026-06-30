use crate::skill_behaviour::SkillBehaviour;

/// Handles armour stat increase on level-up (multi-edition, BB2020+ cap of 11).
///
/// Player-modifier-only behaviour: applies
/// `player.setArmour(min(min(11, position.getArmour()+2), player.getArmour()+1))`.
/// No step hook is registered; `execute_step_hook` is not overridden.
///
/// Note: BB2020+ raises the armour cap from 10 to 11 compared to the BB2016 variant.
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.mixed.ArmourIncreaseBehaviour`.
pub struct ArmourIncreaseBehaviour;

impl ArmourIncreaseBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for ArmourIncreaseBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for ArmourIncreaseBehaviour {
    fn name(&self) -> &'static str { "ArmourIncreaseBehaviour" }
    // Player-modifier-only behaviour — no step hook. execute_step_hook returns false (default).
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = ArmourIncreaseBehaviour::new();
        assert_eq!(b.name(), "ArmourIncreaseBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = ArmourIncreaseBehaviour::default();
        assert_eq!(b.name(), "ArmourIncreaseBehaviour");
    }
}
