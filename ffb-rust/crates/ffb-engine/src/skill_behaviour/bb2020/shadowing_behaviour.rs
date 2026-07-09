use crate::skill_behaviour::SkillBehaviour;

/// BB2020 Shadowing skill behaviour.
///
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2020.ShadowingBehaviour`.
///
/// **BB2020 vs BB2025 differences:**
///
/// 1. **Minimum roll formula:** BB2020 uses a movement-differential formula:
///    `min_roll = max(6 - (shadower_MA - mover_MA), 2)`.
///    BB2025 uses a flat minimum roll of 4.
///
/// 2. **Dialog descriptions:** BB2020 shows the MA-advantage/disadvantage string per shadower in
///    the dialog. BB2025 passes `null` descriptions.
///
/// 3. **No `addShadower` tracking:** BB2020 does not call `gameState.addShadower()` before the roll.
///    BB2025 does to enforce the "each shadower may shadow at most MA times per move" limit.
///
/// 4. **No `movesRandomly` guard:** BB2020 does not skip shadowing for `movesRandomly` players.
///    BB2025 skips shadowing when the mover has `NamedProperties.movesRandomly`.
///
/// 5. **No MA > shadowingCount filter:** BB2020 does not filter shadowers by
///    `shadower.MA > gameState.shadowingCount(shadower)`. BB2025 does.
pub struct ShadowingBehaviour;

impl ShadowingBehaviour {
    pub fn new() -> Self { Self }

    /// Compute the BB2020 minimum roll for a Shadowing attempt.
    ///
    /// Java: `Math.max(6 - moveDifference, 2)` where
    /// `moveDifference = defender_ma - actingPlayer_ma`.
    ///
    /// - `shadower_ma`: movement allowance of the shadowing player (game.getDefender()).
    /// - `mover_ma`: movement allowance of the moving player.
    pub fn minimum_roll_bb2020(shadower_ma: i32, mover_ma: i32) -> i32 {
        let move_difference = shadower_ma - mover_ma;
        (6 - move_difference).max(2)
    }
}

impl Default for ShadowingBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for ShadowingBehaviour {
    fn name(&self) -> &'static str { "ShadowingBehaviour" }

    /// TODO(hook-infra): step-specific state access not yet wired. The pure-logic helper
    /// `minimum_roll_bb2020` encodes the BB2020 MA-differential minimum roll formula.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- pure logic tests for BB2020 minimum roll formula ---

    /// Shadower with equal MA: `max(6 - 0, 2) = 6`.
    #[test]
    fn minimum_roll_equal_ma_is_6() {
        assert_eq!(ShadowingBehaviour::minimum_roll_bb2020(6, 6), 6);
    }

    /// Shadower 2 MA faster: `max(6 - 2, 2) = 4`.
    #[test]
    fn minimum_roll_shadower_faster_reduces_target() {
        assert_eq!(ShadowingBehaviour::minimum_roll_bb2020(8, 6), 4);
    }

    /// Shadower 4 MA faster: `max(6 - 4, 2) = 2` (clamped to minimum 2).
    #[test]
    fn minimum_roll_clamps_to_2() {
        assert_eq!(ShadowingBehaviour::minimum_roll_bb2020(10, 6), 2);
    }

    /// Shadower is slower: `max(6 - (-2), 2) = 8` — very hard to shadow.
    #[test]
    fn minimum_roll_shadower_slower_increases_target() {
        assert_eq!(ShadowingBehaviour::minimum_roll_bb2020(4, 6), 8);
    }

    /// Extreme advantage does not go below 2.
    #[test]
    fn minimum_roll_never_below_2() {
        let roll = ShadowingBehaviour::minimum_roll_bb2020(20, 3);
        assert!(roll >= 2, "minimum roll must be at least 2, got {roll}");
    }

    // --- infrastructure tests ---

    #[test]
    fn name_is_correct() {
        assert_eq!(ShadowingBehaviour::new().name(), "ShadowingBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = ShadowingBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = ShadowingBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
}
