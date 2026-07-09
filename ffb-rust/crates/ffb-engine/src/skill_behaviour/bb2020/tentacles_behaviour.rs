use crate::skill_behaviour::SkillBehaviour;

/// BB2020 Tentacles skill behaviour.
///
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2020.TentaclesBehaviour`.
///
/// **BB2020 vs BB2025 difference:**
///
/// BB2020 applies Tentacles when the mover is dodging, jumping, OR has already blocked AND
/// `coordinateFrom` is non-null:
/// ```java
/// if (actingPlayer.isDodging() || actingPlayer.isJumping()
///     || (actingPlayer.hasBlocked() && state.coordinateFrom != null)) {
/// ```
///
/// BB2025 removes the `hasBlocked` branch entirely:
/// ```java
/// if (actingPlayer.isDodging() || actingPlayer.isJumping()) {
/// ```
///
/// This means in BB2020, Tentacles can also trigger during a blitz-move sequence when the attacker
/// has already blocked (the `hasBlocked` path). BB2025 restricts Tentacles to dodge and jump only.
pub struct TentaclesBehaviour;

impl TentaclesBehaviour {
    pub fn new() -> Self { Self }

    /// Returns `true` when Tentacles can trigger during a blitz-move (attacker has already blocked)
    /// in this edition. BB2020: `true`. BB2025: `false`.
    pub const fn triggers_on_post_block_move() -> bool {
        true
    }

    /// Evaluate whether Tentacles should trigger in **BB2020**.
    ///
    /// - `is_dodging`: `actingPlayer.isDodging()`
    /// - `is_jumping`: `actingPlayer.isJumping()`
    /// - `has_blocked`: `actingPlayer.hasBlocked()`
    /// - `coordinate_from_is_some`: `state.coordinateFrom != null`
    pub fn should_apply_bb2020(
        is_dodging: bool,
        is_jumping: bool,
        has_blocked: bool,
        coordinate_from_is_some: bool,
    ) -> bool {
        is_dodging || is_jumping || (has_blocked && coordinate_from_is_some)
    }
}

impl Default for TentaclesBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for TentaclesBehaviour {
    fn name(&self) -> &'static str { "TentaclesBehaviour" }

    /// TODO(hook-infra): step-specific state access not yet wired.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// BB2020: Tentacles triggers when player has blocked and coordinateFrom is set.
    #[test]
    fn bb2020_triggers_on_post_block_move() {
        assert!(TentaclesBehaviour::should_apply_bb2020(false, false, true, true));
    }

    /// BB2020: Tentacles does NOT trigger when player has blocked but coordinateFrom is null.
    #[test]
    fn bb2020_does_not_trigger_post_block_without_coordinate_from() {
        assert!(!TentaclesBehaviour::should_apply_bb2020(false, false, true, false));
    }

    /// BB2020: Tentacles triggers on dodge (same as BB2025).
    #[test]
    fn bb2020_triggers_on_dodge() {
        assert!(TentaclesBehaviour::should_apply_bb2020(true, false, false, false));
    }

    /// BB2020: Tentacles triggers on jump (same as BB2025).
    #[test]
    fn bb2020_triggers_on_jump() {
        assert!(TentaclesBehaviour::should_apply_bb2020(false, true, false, false));
    }

    /// triggers_on_post_block_move constant is true for BB2020.
    #[test]
    fn triggers_on_post_block_move_constant_is_true() {
        assert!(TentaclesBehaviour::triggers_on_post_block_move());
    }

    #[test]
    fn name_is_correct() {
        assert_eq!(TentaclesBehaviour::new().name(), "TentaclesBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = TentaclesBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = TentaclesBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, before);
    }
}
