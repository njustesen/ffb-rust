use crate::skill_behaviour::SkillBehaviour;

/// BB2020 BloodLust skill behaviour.
///
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2020.BloodLustBehaviour`.
///
/// **BB2020 vs BB2025 differences:**
///
/// 1. **`changeToMove` action set:** BB2020 includes `BLITZ_MOVE` and `GAZE_MOVE` in the set of
///    actions that switch to a MOVE variant on blood-lust failure. BB2025 removes them and adds
///    `SECURE_THE_BALL`.
///
/// 2. **`waitForActionChange` trigger set:** BB2020 includes `BLITZ_MOVE` and `GAZE_MOVE` in the
///    set of actions that trigger `WAIT_FOR_ACTION_CHANGE` processing. BB2025 removes them and adds
///    `SECURE_THE_BALL` and `PUNT`.
///
/// All bite mechanics, re-roll handling, and dialog logic are identical between editions.
pub struct BloodLustBehaviour;

impl BloodLustBehaviour {
    pub fn new() -> Self { Self }

    /// Returns `true` when the player action kind should cause the player to switch to a MOVE
    /// variant on blood-lust failure, using **BB2020 rules**.
    ///
    /// Java (BB2020):
    /// ```java
    /// boolean changeToMove = Arrays.asList(new PlayerAction[]{
    ///     VICIOUS_VINES, BLOCK, THROW_BOMB, STAND_UP, BLITZ_MOVE, GAZE_MOVE, MULTIPLE_BLOCK
    /// }).contains(actingPlayer.getPlayerAction());
    /// ```
    pub fn change_to_move_bb2020(action: BloodLustActionKind) -> bool {
        matches!(
            action,
            BloodLustActionKind::ViciousVines
                | BloodLustActionKind::Block
                | BloodLustActionKind::ThrowBomb
                | BloodLustActionKind::StandUp
                | BloodLustActionKind::BlitzMove   // BB2020 only
                | BloodLustActionKind::GazeMove    // BB2020 only
                | BloodLustActionKind::MultipleBlock
        )
    }

    /// Returns `true` when the player action kind triggers the `WAIT_FOR_ACTION_CHANGE` dispatch
    /// path (wait-for-action-change action set) in **BB2020**.
    ///
    /// Java (BB2020):
    /// ```java
    /// {VICIOUS_VINES, BLOCK, PASS, HAND_OVER, THROW_BOMB, THROW_TEAM_MATE, KICK_TEAM_MATE,
    ///  FOUL, STAND_UP, STAND_UP_BLITZ, BLITZ_MOVE, GAZE_MOVE, MULTIPLE_BLOCK}
    /// ```
    pub fn wait_for_action_change_bb2020(action: BloodLustActionKind) -> bool {
        matches!(
            action,
            BloodLustActionKind::ViciousVines
                | BloodLustActionKind::Block
                | BloodLustActionKind::Pass
                | BloodLustActionKind::HandOver
                | BloodLustActionKind::ThrowBomb
                | BloodLustActionKind::ThrowTeamMate
                | BloodLustActionKind::KickTeamMate
                | BloodLustActionKind::Foul
                | BloodLustActionKind::StandUp
                | BloodLustActionKind::StandUpBlitz
                | BloodLustActionKind::BlitzMove   // BB2020 only
                | BloodLustActionKind::GazeMove    // BB2020 only
                | BloodLustActionKind::MultipleBlock
        )
    }
}

/// Thin classification of player action kinds relevant to BloodLust's action-change logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BloodLustActionKind {
    ViciousVines,
    Block,
    Pass,
    HandOver,
    ThrowBomb,
    ThrowTeamMate,
    KickTeamMate,
    Foul,
    StandUp,
    StandUpBlitz,
    MultipleBlock,
    /// BB2020 only — included in changeToMove and waitForActionChange.
    BlitzMove,
    /// BB2020 only — included in changeToMove and waitForActionChange.
    GazeMove,
    /// BB2025 only — SecureTheBall (absent in BB2020).
    SecureTheBall,
    /// BB2025 only — Punt (absent in BB2020).
    Punt,
    Other,
}

impl Default for BloodLustBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for BloodLustBehaviour {
    fn name(&self) -> &'static str { "BloodLustBehaviour" }

    /// TODO(hook-infra): step-specific state access not yet wired. The pure-logic helpers
    /// `change_to_move_bb2020` and `wait_for_action_change_bb2020` encode the BB2020 action sets.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- BB2020 changeToMove set tests ---

    /// BB2020: BlitzMove is in the changeToMove set.
    #[test]
    fn blitz_move_in_change_to_move_bb2020() {
        assert!(
            BloodLustBehaviour::change_to_move_bb2020(BloodLustActionKind::BlitzMove),
            "BB2020: BlitzMove should be in changeToMove set"
        );
    }

    /// BB2020: GazeMove is in the changeToMove set.
    #[test]
    fn gaze_move_in_change_to_move_bb2020() {
        assert!(
            BloodLustBehaviour::change_to_move_bb2020(BloodLustActionKind::GazeMove),
            "BB2020: GazeMove should be in changeToMove set"
        );
    }

    /// BB2020: SecureTheBall is NOT in the changeToMove set (BB2025-only).
    #[test]
    fn secure_the_ball_not_in_change_to_move_bb2020() {
        assert!(
            !BloodLustBehaviour::change_to_move_bb2020(BloodLustActionKind::SecureTheBall),
            "BB2020: SecureTheBall must NOT be in changeToMove set"
        );
    }

    // --- BB2020 waitForActionChange set tests ---

    /// BB2020: BlitzMove is in the waitForActionChange set.
    #[test]
    fn blitz_move_in_wait_for_action_change_bb2020() {
        assert!(
            BloodLustBehaviour::wait_for_action_change_bb2020(BloodLustActionKind::BlitzMove),
            "BB2020: BlitzMove should be in waitForActionChange set"
        );
    }

    /// BB2020: GazeMove is in the waitForActionChange set.
    #[test]
    fn gaze_move_in_wait_for_action_change_bb2020() {
        assert!(
            BloodLustBehaviour::wait_for_action_change_bb2020(BloodLustActionKind::GazeMove)
        );
    }

    /// BB2020: Punt is NOT in the waitForActionChange set (BB2025-only).
    #[test]
    fn punt_not_in_wait_for_action_change_bb2020() {
        assert!(
            !BloodLustBehaviour::wait_for_action_change_bb2020(BloodLustActionKind::Punt),
            "BB2020: Punt must NOT be in waitForActionChange set"
        );
    }

    // --- infrastructure tests ---

    #[test]
    fn name_is_correct() {
        assert_eq!(BloodLustBehaviour::new().name(), "BloodLustBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = BloodLustBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = BloodLustBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
}
