use crate::skill_behaviour::SkillBehaviour;

/// Wild Animal: player must block or blitz or be wasted (like BoneHead but with different
/// goodConditions). goodConditions = action is BLITZ_MOVE / BLITZ / BLOCK / MULTIPLE_BLOCK /
/// STAND_UP_BLITZ. On cancel: sets player to STANDING (not PRONE) + inactive, plays ROAR sound.
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2016.WildAnimalBehaviour`.
pub struct WildAnimalBehaviour;

impl WildAnimalBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for WildAnimalBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for WildAnimalBehaviour {
    fn name(&self) -> &'static str { "WildAnimalBehaviour" }

    /// Java logic (handleExecuteStepHook — mirrors BoneHeadBehaviour with different conditions):
    ///   1. Determine goodConditions: current PlayerAction is one of BLITZ_MOVE, BLITZ, BLOCK,
    ///      MULTIPLE_BLOCK, STAND_UP_BLITZ.
    ///   2. If goodConditions: roll confusion (minimumRollConfusion(goodConditions)).
    ///      On success: player acts normally.
    ///   3. If goodConditions is false OR roll fails:
    ///      a. Set player state to STANDING (not PRONE — differs from BoneHead).
    ///      b. Set player inactive (cancelPlayerAction()).
    ///      c. Play ROAR sound effect.
    ///      d. Check for reroll; set StepState.status accordingly.
    ///   4. Reads/writes: StepState.status, StepState.reRolledAction,
    ///      StepState.confusionRoll, StepState.playerAction.
    ///
    // TODO(hook-infra): step-specific state (StepState.status)
    // TODO(hook-infra): step-specific state (StepState.reRolledAction)
    // TODO(hook-infra): step-specific state (StepState.confusionRoll)
    // TODO(hook-infra): step-specific state (StepState.playerAction)
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = WildAnimalBehaviour::new();
        assert_eq!(b.name(), "WildAnimalBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = WildAnimalBehaviour::default();
        assert_eq!(b.name(), "WildAnimalBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = WildAnimalBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2016,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = WildAnimalBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
}
