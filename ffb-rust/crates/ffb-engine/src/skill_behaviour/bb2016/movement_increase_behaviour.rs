use crate::skill_behaviour::SkillBehaviour;

/// Handles movement stat increase on level-up.
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2016.MovementIncreaseBehaviour`.
///
/// Player modifier only: `player.setMovement(min(min(10, position.getMovement()+2), player.getMovement()+1))`.
/// No step-hook logic — `execute_step_hook` is a no-op.
pub struct MovementIncreaseBehaviour;

impl MovementIncreaseBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for MovementIncreaseBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for MovementIncreaseBehaviour {
    fn name(&self) -> &'static str { "MovementIncreaseBehaviour" }

    /// No-op: MovementIncreaseBehaviour is a player-modifier-only behaviour.
    /// The Java class only overrides `getPlayerModifiers()`, not `handleExecuteStepHook`.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
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
