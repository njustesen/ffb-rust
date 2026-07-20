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

// Java-derived cases (MovementCalcTest) live in the 1:1 mirror module
// ffb-engine/src/util/movement_calc.rs; the former tests here duplicated
// them (same inputs and expectations) and were removed. Only cases the
// mirror does not cover remain.
#[cfg(test)]
mod tests {
    use super::*;

    // ── is_next_move_gfi (covers currentMove 1–4 at MA 6, not in the mirror) ──

    #[test]
    fn no_gfi_before_ma_is_reached() {
        for current in 0..6 {
            assert!(!is_next_move_gfi(current, 6), "current={current}");
        }
    }
}
