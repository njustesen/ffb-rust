// 1:1 translation of com.fumbbl.ffb.server.util.MovementCalc

pub struct MovementCalc;

impl MovementCalc {
    /// Going For It adds this many extra squares beyond MA by default.
    pub const STANDARD_GFI_SQUARES: i32 = 2;

    /// Standing up from prone costs this many movement squares.
    pub const STAND_UP_COST: i32 = 3;

    /// Minimum roll required for any GFI attempt (all editions).
    pub const GFI_MINIMUM_ROLL: i32 = 2;

    pub fn new() -> Self {
        Self
    }

    /// Maximum squares a player may move this action.
    pub fn max_movement(ma: i32, gfi_squares: i32) -> i32 {
        ma + gfi_squares
    }

    /// Whether the player's next square requires a GFI roll.
    /// GFI triggers as soon as currentMove equals or exceeds MA.
    pub fn is_next_move_going_for_it(current_move: i32, ma: i32) -> bool {
        current_move >= ma
    }

    /// Whether a prone player with the given MA must roll to stand up.
    /// Standing up requires a 4+ roll if MA ≤ 3 (cost of standing = 3 squares ≥ MA).
    pub fn must_roll_to_stand_up(ma: i32) -> bool {
        ma <= Self::STAND_UP_COST
    }

    /// Whether the player has movement remaining (including potential GFI).
    pub fn has_move_left(current_move: i32, ma: i32, gfi_squares: i32) -> bool {
        current_move < ma + gfi_squares
    }

    /// GFI squares for a standard action: base 2, optionally +1 if player has an Extra GFI skill.
    pub fn gfi_squares(has_extra_gfi: bool) -> i32 {
        Self::STANDARD_GFI_SQUARES + if has_extra_gfi { 1 } else { 0 }
    }
}

impl Default for MovementCalc {
    fn default() -> Self {
        Self::new()
    }
}

// Tests mirror ffb-java/ffb-server/src/test/java/com/fumbbl/ffb/server/util/MovementCalcTest.java 1:1
#[cfg(test)]
mod tests {
    use super::*;

    // ── max_movement ──────────────────────────────────────────────────────────

    #[test]
    fn max_movement_no_gfi_equals_ma() {
        assert_eq!(6, MovementCalc::max_movement(6, 0));
        assert_eq!(4, MovementCalc::max_movement(4, 0));
    }

    #[test]
    fn max_movement_standard_gfi_ma_plus2() {
        assert_eq!(8, MovementCalc::max_movement(6, MovementCalc::STANDARD_GFI_SQUARES));
        assert_eq!(6, MovementCalc::max_movement(4, MovementCalc::STANDARD_GFI_SQUARES));
    }

    #[test]
    fn max_movement_extra_gfi_ma_plus3() {
        assert_eq!(9, MovementCalc::max_movement(6, 3));
    }

    #[test]
    fn max_movement_with_temporary_modifier_applied_to_ma() {
        // MA 6, temporary +1 from skill → MA is passed as 7
        assert_eq!(9, MovementCalc::max_movement(7, MovementCalc::STANDARD_GFI_SQUARES));
    }

    // ── is_next_move_going_for_it ─────────────────────────────────────────────

    #[test]
    fn is_next_move_going_for_it_various_cases() {
        for (current_move, ma, expected) in [
            (0, 6, false),
            (5, 6, false),
            (6, 6, true), // exactly at MA → next is GFI
            (7, 6, true), // beyond MA → still GFI
            (0, 4, false),
            (3, 4, false),
            (4, 4, true),
            (5, 4, true),
        ] {
            assert_eq!(
                expected,
                MovementCalc::is_next_move_going_for_it(current_move, ma),
                "currentMove={current_move} ma={ma}"
            );
        }
    }

    #[test]
    fn is_next_move_going_for_it_ma1_gfi_immediately_after_first_move() {
        // Snotling with MA 1: first GFI after move 1
        assert!(!MovementCalc::is_next_move_going_for_it(0, 1));
        assert!(MovementCalc::is_next_move_going_for_it(1, 1));
    }

    // ── must_roll_to_stand_up ─────────────────────────────────────────────────

    #[test]
    fn must_roll_to_stand_up_ma3_or_under_requires_roll() {
        for (ma, expected) in [
            (1, true),
            (2, true),
            (3, true),
            (4, false),
            (5, false),
            (6, false),
            (9, false),
        ] {
            assert_eq!(expected, MovementCalc::must_roll_to_stand_up(ma), "ma={ma}");
        }
    }

    #[test]
    fn must_roll_to_stand_up_boundary_exactly3_must_roll() {
        assert!(MovementCalc::must_roll_to_stand_up(3));
    }

    #[test]
    fn must_roll_to_stand_up_boundary_4_no_roll() {
        assert!(!MovementCalc::must_roll_to_stand_up(4));
    }

    // ── has_move_left ─────────────────────────────────────────────────────────

    #[test]
    fn has_move_left_not_yet_moved_always_has_move() {
        assert!(MovementCalc::has_move_left(0, 6, 0));
    }

    #[test]
    fn has_move_left_reached_ma_exact_no_move_without_gfi() {
        assert!(!MovementCalc::has_move_left(6, 6, 0));
    }

    #[test]
    fn has_move_left_reached_ma_has_gfi_squares_left() {
        assert!(MovementCalc::has_move_left(6, 6, MovementCalc::STANDARD_GFI_SQUARES));
    }

    #[test]
    fn has_move_left_used_all_gfi_no_move_left() {
        // MA 6 + 2 GFI = 8 total; after moving 8 → no move left
        assert!(!MovementCalc::has_move_left(8, 6, MovementCalc::STANDARD_GFI_SQUARES));
    }

    // ── constants ─────────────────────────────────────────────────────────────

    #[test]
    fn constants_values_are_correct() {
        assert_eq!(2, MovementCalc::STANDARD_GFI_SQUARES);
        assert_eq!(3, MovementCalc::STAND_UP_COST);
        assert_eq!(2, MovementCalc::GFI_MINIMUM_ROLL);
    }

    #[test]
    fn gfi_squares_no_skill_returns2() {
        assert_eq!(2, MovementCalc::gfi_squares(false));
    }

    #[test]
    fn gfi_squares_with_extra_gfi_returns3() {
        assert_eq!(3, MovementCalc::gfi_squares(true));
    }

    // ── interaction: GFI minimum roll ─────────────────────────────────────────

    #[test]
    fn gfi_minimum_roll_is_always2_regardless_of_stats() {
        // GFI always requires 2+ (not agility-based), same across all editions
        assert_eq!(2, MovementCalc::GFI_MINIMUM_ROLL);
    }
}
