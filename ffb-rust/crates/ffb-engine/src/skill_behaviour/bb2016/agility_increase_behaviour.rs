use crate::skill_behaviour::SkillBehaviour;

/// Handles agility stat increase on level-up.
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2016.AgilityIncreaseBehaviour`.
pub struct AgilityIncreaseBehaviour;

impl AgilityIncreaseBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for AgilityIncreaseBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for AgilityIncreaseBehaviour {
    fn name(&self) -> &'static str { "AgilityIncreaseBehaviour" }

    /// Player-modifier-only behaviour — no step hook in Java source.
    ///
    /// Java `AgilityIncreaseBehaviour` only registers a player modifier:
    ///   `player.setAgility(min(min(8, position.getAgility() + 2), player.getAgility() + 1))`
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
        let b = AgilityIncreaseBehaviour::new();
        assert_eq!(b.name(), "AgilityIncreaseBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = AgilityIncreaseBehaviour::default();
        assert_eq!(b.name(), "AgilityIncreaseBehaviour");
    }
}
