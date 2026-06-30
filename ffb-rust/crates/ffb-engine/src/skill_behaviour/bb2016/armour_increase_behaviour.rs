use crate::skill_behaviour::SkillBehaviour;

/// Handles armour stat increase on level-up.
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2016.ArmourIncreaseBehaviour`.
pub struct ArmourIncreaseBehaviour;

impl ArmourIncreaseBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for ArmourIncreaseBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for ArmourIncreaseBehaviour {
    fn name(&self) -> &'static str { "ArmourIncreaseBehaviour" }

    /// Player-modifier-only behaviour — no step hook in Java source.
    ///
    /// Java `ArmourIncreaseBehaviour` only registers a player modifier:
    ///   `player.setArmour(min(min(10, position.getArmour() + 2), player.getArmour() + 1))`
    /// There is no `StepModifier` registered, so this method is a no-op.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
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
