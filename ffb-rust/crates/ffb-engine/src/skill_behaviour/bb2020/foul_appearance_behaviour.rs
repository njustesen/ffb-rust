use crate::skill_behaviour::SkillBehaviour;

/// BB2020 FoulAppearance skill behaviour.
///
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2020.FoulAppearanceBehaviour`.
///
/// **BB2020 vs BB2025 difference (handleFailure method):**
///
/// In BB2025 the `END_PLAYER_ACTION` step-parameter is published for blitzing actions too
/// (adds `playerAction.isBlitzing()`). In BB2020 it is only published for:
/// - `PlayerAction.GAZE`
/// - Actions where `playerAction.isBlockAction()` is true.
///
/// All other logic (roll mechanics, minimum roll, re-roll handling) is identical.
pub struct FoulAppearanceBehaviour;

impl FoulAppearanceBehaviour {
    pub fn new() -> Self { Self }

    /// Decide whether END_PLAYER_ACTION should be published for the given action kind in **BB2020**.
    ///
    /// Java:
    /// ```java
    /// if (playerAction == PlayerAction.GAZE || (playerAction != null && playerAction.isBlockAction())) {
    ///     step.publishParameter(StepParameter.from(StepParameterKey.END_PLAYER_ACTION, true));
    /// }
    /// ```
    pub fn should_publish_end_player_action_bb2020(
        is_gaze: bool,
        is_block_action: bool,
    ) -> bool {
        is_gaze || is_block_action
    }
}

impl Default for FoulAppearanceBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for FoulAppearanceBehaviour {
    fn name(&self) -> &'static str { "FoulAppearanceBehaviour" }

    /// TODO(hook-infra): step-specific state access not yet wired. The pure-logic helper
    /// `should_publish_end_player_action_bb2020` encodes the BB2020 failure-handler condition.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- BB2020 END_PLAYER_ACTION condition tests ---

    /// BB2020: GAZE action triggers END_PLAYER_ACTION.
    #[test]
    fn gaze_action_triggers_end_player_action_bb2020() {
        assert!(
            FoulAppearanceBehaviour::should_publish_end_player_action_bb2020(true, false),
            "BB2020: GAZE should publish END_PLAYER_ACTION"
        );
    }

    /// BB2020: block action triggers END_PLAYER_ACTION.
    #[test]
    fn block_action_triggers_end_player_action_bb2020() {
        assert!(
            FoulAppearanceBehaviour::should_publish_end_player_action_bb2020(false, true),
            "BB2020: block action should publish END_PLAYER_ACTION"
        );
    }

    /// BB2020: blitz action does NOT trigger END_PLAYER_ACTION (unlike BB2025 which adds isBlitzing).
    /// Represented as neither gaze nor block-action being true.
    #[test]
    fn blitz_only_action_does_not_trigger_end_player_action_bb2020() {
        assert!(
            !FoulAppearanceBehaviour::should_publish_end_player_action_bb2020(false, false),
            "BB2020: blitz-only action must NOT publish END_PLAYER_ACTION"
        );
    }

    /// Move action does not trigger END_PLAYER_ACTION.
    #[test]
    fn move_action_does_not_trigger_end_player_action() {
        assert!(
            !FoulAppearanceBehaviour::should_publish_end_player_action_bb2020(false, false)
        );
    }

    // --- infrastructure tests ---

    #[test]
    fn name_is_correct() {
        assert_eq!(FoulAppearanceBehaviour::new().name(), "FoulAppearanceBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = FoulAppearanceBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = FoulAppearanceBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
}
