use crate::skill_behaviour::SkillBehaviour;

/// Handles movement stat increase on level-up (multi-edition, BB2020+ cap of 9).
///
/// Player-modifier-only behaviour: applies
/// `player.setMovement(min(min(9, position.getMovement()+2), player.getMovement()+1))`.
/// No step hook is registered; `execute_step_hook` is not overridden.
///
/// Note: BB2020+ lowers the movement cap from 10 to 9 compared to the BB2016 variant.
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.mixed.MovementIncreaseBehaviour`.
pub struct MovementIncreaseBehaviour;

impl MovementIncreaseBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for MovementIncreaseBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for MovementIncreaseBehaviour {
    fn name(&self) -> &'static str { "MovementIncreaseBehaviour" }
    // Player-modifier-only behaviour — no step hook. execute_step_hook returns false (default).
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = MovementIncreaseBehaviour::new();
        assert_eq!(b.name(), "MovementIncreaseBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = MovementIncreaseBehaviour::default();
        assert_eq!(b.name(), "MovementIncreaseBehaviour");
    }
}
