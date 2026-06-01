/// Going For It adds this many extra squares beyond MA by default.
pub const STANDARD_GFI_SQUARES: i32 = 2;

/// Standing up from prone costs this many movement squares.
pub const STAND_UP_COST: i32 = 3;

/// Minimum roll required for any GFI attempt (all editions, all modifiers).
pub const GFI_MINIMUM_ROLL: i32 = 2;

/// Maximum squares a player may move in a single action.
///
/// `gfi_squares` is 0 when not using Going For It, `STANDARD_GFI_SQUARES` (2) normally,
/// or 3+ when the player has Extra GFI skills.
pub fn max_movement_allowance(ma: i32, gfi_squares: i32) -> i32 {
    ma + gfi_squares
}

/// Whether the player's next square requires a GFI roll.
///
/// GFI triggers as soon as `current_move >= ma`.
pub fn is_next_move_gfi(current_move: i32, ma: i32) -> bool {
    current_move >= ma
}

/// Whether a prone player with the given MA must roll (4+) to stand up.
///
/// Standing up costs `STAND_UP_COST` (3) squares; if MA ≤ 3 the full cost
/// is at or beyond the player's allowance, so a roll is required.
pub fn must_roll_to_stand_up(ma: i32) -> bool {
    ma <= STAND_UP_COST
}

/// Whether the player has movement squares remaining.
///
/// Pass `gfi_squares = 0` when GFI is not available; `STANDARD_GFI_SQUARES` when it is.
pub fn has_move_left(current_move: i32, ma: i32, gfi_squares: i32) -> bool {
    current_move < ma + gfi_squares
}

/// GFI squares available based on skill flags.
pub fn gfi_squares(has_extra_gfi: bool) -> i32 {
    STANDARD_GFI_SQUARES + if has_extra_gfi { 1 } else { 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── max_movement_allowance ────────────────────────────────────────────────

    #[test]
    fn no_gfi_equals_ma() {
        assert_eq!(max_movement_allowance(6, 0), 6);
        assert_eq!(max_movement_allowance(4, 0), 4);
    }

    #[test]
    fn standard_gfi_is_ma_plus_2() {
        assert_eq!(max_movement_allowance(6, STANDARD_GFI_SQUARES), 8);
        assert_eq!(max_movement_allowance(4, STANDARD_GFI_SQUARES), 6);
    }

    #[test]
    fn extra_gfi_skill_is_ma_plus_3() {
        assert_eq!(max_movement_allowance(6, 3), 9);
    }

    // ── is_next_move_gfi ──────────────────────────────────────────────────────

    #[test]
    fn gfi_triggers_exactly_at_ma() {
        assert!(!is_next_move_gfi(5, 6));
        assert!(is_next_move_gfi(6, 6));
    }

    #[test]
    fn gfi_triggers_beyond_ma() {
        assert!(is_next_move_gfi(7, 6));
        assert!(is_next_move_gfi(8, 6));
    }

    #[test]
    fn no_gfi_before_ma_is_reached() {
        for current in 0..6 {
            assert!(!is_next_move_gfi(current, 6), "current={current}");
        }
    }

    #[test]
    fn ma1_player_gfi_immediately_after_first_square() {
        assert!(!is_next_move_gfi(0, 1));
        assert!(is_next_move_gfi(1, 1));
    }

    // ── must_roll_to_stand_up ─────────────────────────────────────────────────

    #[test]
    fn ma_1_to_3_must_roll() {
        assert!(must_roll_to_stand_up(1));
        assert!(must_roll_to_stand_up(2));
        assert!(must_roll_to_stand_up(3));
    }

    #[test]
    fn ma_4_and_above_no_roll() {
        assert!(!must_roll_to_stand_up(4));
        assert!(!must_roll_to_stand_up(6));
        assert!(!must_roll_to_stand_up(9));
    }

    // ── has_move_left ─────────────────────────────────────────────────────────

    #[test]
    fn move_left_not_yet_moved() {
        assert!(has_move_left(0, 6, 0));
    }

    #[test]
    fn no_move_left_at_ma_without_gfi() {
        assert!(!has_move_left(6, 6, 0));
    }

    #[test]
    fn move_left_at_ma_with_gfi() {
        assert!(has_move_left(6, 6, STANDARD_GFI_SQUARES));
    }

    #[test]
    fn no_move_left_after_all_gfi() {
        assert!(!has_move_left(8, 6, STANDARD_GFI_SQUARES));
    }

    // ── constants ─────────────────────────────────────────────────────────────

    #[test]
    fn standard_gfi_is_2() {
        assert_eq!(STANDARD_GFI_SQUARES, 2);
    }

    #[test]
    fn stand_up_cost_is_3() {
        assert_eq!(STAND_UP_COST, 3);
    }

    #[test]
    fn gfi_minimum_roll_is_2() {
        assert_eq!(GFI_MINIMUM_ROLL, 2);
    }

    #[test]
    fn gfi_squares_no_skill_is_2() {
        assert_eq!(gfi_squares(false), 2);
    }

    #[test]
    fn gfi_squares_with_extra_gfi_is_3() {
        assert_eq!(gfi_squares(true), 3);
    }

    #[test]
    fn max_movement_with_temporary_ma_modifier() {
        // Caller passes ma=7 (6 base + 1 temporary boost); standard GFI → 9
        assert_eq!(max_movement_allowance(7, STANDARD_GFI_SQUARES), 9);
    }

    #[test]
    fn gfi_minimum_roll_is_always_2() {
        assert_eq!(GFI_MINIMUM_ROLL, 2);
    }
}
