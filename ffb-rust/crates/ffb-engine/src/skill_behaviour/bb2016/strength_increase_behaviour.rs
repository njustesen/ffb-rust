use crate::skill_behaviour::SkillBehaviour;

/// Handles strength stat increase on level-up.
/// Only player modifier: `player.setStrength(min(min(10, position.getStrength()+2),
/// player.getStrength()+1))`. No execute_step_hook logic.
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2016.StrengthIncreaseBehaviour`.
pub struct StrengthIncreaseBehaviour;

impl StrengthIncreaseBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for StrengthIncreaseBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for StrengthIncreaseBehaviour {
    fn name(&self) -> &'static str { "StrengthIncreaseBehaviour" }

    /// Java: only provides a player modifier (`getPlayerModifiers`):
    ///   `player.setStrength(min(min(10, position.getStrength() + 2), player.getStrength() + 1))`
    /// No `handleExecuteStepHook` override — nothing to do here.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = StrengthIncreaseBehaviour::new();
        assert_eq!(b.name(), "StrengthIncreaseBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = StrengthIncreaseBehaviour::default();
        assert_eq!(b.name(), "StrengthIncreaseBehaviour");
    }
}
