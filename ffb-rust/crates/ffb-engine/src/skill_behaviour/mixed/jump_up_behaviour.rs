use crate::skill_behaviour::SkillBehaviour;

/// Jump Up (mixed/BB2020+ version): player may stand up without spending movement,
/// but must pass an agility check; uses `JumpUpModifierFactory` for modifiers.
///
/// Registers on StepStandUp (BLOCK / MULTIPLE_BLOCK action path).
///
/// Java `execute_step_hook` logic:
/// 1. Only fires when the action is `BLOCK` or `MULTIPLE_BLOCK`.
/// 2. Gather agility modifiers from `JumpUpModifierFactory`.
/// 3. Roll the agility jump-up check (target derived from player agility + modifiers).
/// 4. On **fail**:
///    - Set the player to prone.
///    - Set next action to `END_PLAYER_ACTION`.
///    - GOTO `StepState.goToLabelOnFailure`.
/// 5. On **success**: player stands up for free, continue normally.
/// 6. Supports re-rolling via `ReRollSource::JUMP_UP` if available.
///
/// All step-local state fields are unavailable in the current Rust signature:
// TODO(hook-infra): step-specific state (StepState.goToLabelOnFailure)
// TODO(hook-infra): step reroll fields (ReRollSource::JUMP_UP)
///
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.mixed.JumpUpBehaviour`.
pub struct JumpUpBehaviour;

impl JumpUpBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for JumpUpBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for JumpUpBehaviour {
    fn name(&self) -> &'static str { "JumpUpBehaviour" }

    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = JumpUpBehaviour::new();
        assert_eq!(b.name(), "JumpUpBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = JumpUpBehaviour::default();
        assert_eq!(b.name(), "JumpUpBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = JumpUpBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2025,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = JumpUpBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
    #[test]
    fn default_creates_instance_same_as_new() {
        let _a = JumpUpBehaviour::new();
        let _b = JumpUpBehaviour::default();
    }
}
