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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_movement_is_ma_plus_gfi() {
        assert_eq!(MovementCalc::max_movement(6, 2), 8);
        assert_eq!(MovementCalc::max_movement(6, 0), 6);
    }

    #[test]
    fn is_next_move_going_for_it_exactly_at_ma() {
        // When current_move == ma, the next square is GFI
        assert!(MovementCalc::is_next_move_going_for_it(6, 6));
    }

    #[test]
    fn is_not_going_for_it_below_ma() {
        assert!(!MovementCalc::is_next_move_going_for_it(5, 6));
    }

    #[test]
    fn is_going_for_it_above_ma() {
        assert!(MovementCalc::is_next_move_going_for_it(7, 6));
    }

    #[test]
    fn must_roll_to_stand_up_ma_3() {
        assert!(MovementCalc::must_roll_to_stand_up(3));
    }

    #[test]
    fn must_roll_to_stand_up_ma_2() {
        assert!(MovementCalc::must_roll_to_stand_up(2));
    }

    #[test]
    fn must_not_roll_to_stand_up_ma_4() {
        assert!(!MovementCalc::must_roll_to_stand_up(4));
    }

    #[test]
    fn has_move_left_when_not_exhausted() {
        assert!(MovementCalc::has_move_left(3, 6, 2));
    }

    #[test]
    fn no_move_left_when_exhausted() {
        assert!(!MovementCalc::has_move_left(8, 6, 2));
    }

    #[test]
    fn no_move_left_at_exact_limit() {
        assert!(!MovementCalc::has_move_left(8, 6, 2));
    }

    #[test]
    fn gfi_squares_without_extra() {
        assert_eq!(MovementCalc::gfi_squares(false), 2);
    }

    #[test]
    fn gfi_squares_with_extra() {
        assert_eq!(MovementCalc::gfi_squares(true), 3);
    }

    #[test]
    fn stand_up_cost_constant_is_3() {
        assert_eq!(MovementCalc::STAND_UP_COST, 3);
    }

    #[test]
    fn gfi_minimum_roll_constant_is_2() {
        assert_eq!(MovementCalc::GFI_MINIMUM_ROLL, 2);
    }
}
