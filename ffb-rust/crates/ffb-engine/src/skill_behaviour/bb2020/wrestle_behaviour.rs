use crate::skill_behaviour::SkillBehaviour;

/// BB2020 Wrestle skill behaviour.
///
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2020.WrestleBehaviour`.
///
/// **BB2020 vs BB2025 difference:**
///
/// BB2025 publishes a `REVERT_END_TURN = true` step parameter when the Wrestle user (attacker)
/// has the ball:
/// ```java
/// if (UtilPlayer.hasBall(game, actingPlayer.getPlayer())) {
///     step.publishParameter(StepParameter.from(StepParameterKey.REVERT_END_TURN, true));
/// }
/// ```
/// This reverts the end-of-turn state when the ball carrier wrestles (preventing accidental
/// turn-end state pollution). BB2020 does not have this parameter.
pub struct WrestleBehaviour;

impl WrestleBehaviour {
    pub fn new() -> Self { Self }

    /// Returns `true` when `REVERT_END_TURN = true` should be published if the Wrestle user has
    /// the ball (BB2025 feature). BB2020 always returns `false`.
    pub fn publish_revert_end_turn_when_attacker_has_ball_bb2020() -> bool {
        false
    }
}

impl Default for WrestleBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for WrestleBehaviour {
    fn name(&self) -> &'static str { "WrestleBehaviour" }

    /// TODO(hook-infra): step-specific state access not yet wired.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// BB2020 does not publish REVERT_END_TURN when attacker has the ball.
    #[test]
    fn bb2020_does_not_publish_revert_end_turn() {
        assert!(!WrestleBehaviour::publish_revert_end_turn_when_attacker_has_ball_bb2020());
    }

    /// The function returns false regardless of context in BB2020.
    #[test]
    fn revert_end_turn_always_false_in_bb2020() {
        assert_eq!(
            WrestleBehaviour::publish_revert_end_turn_when_attacker_has_ball_bb2020(),
            false
        );
    }

    #[test]
    fn name_is_correct() {
        assert_eq!(WrestleBehaviour::new().name(), "WrestleBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = WrestleBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = WrestleBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, before);
    }
}
