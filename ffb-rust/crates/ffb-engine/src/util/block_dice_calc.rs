// 1:1 translation of com.fumbbl.ffb.server.util.BlockDiceCalc

pub struct BlockDiceCalc;

impl BlockDiceCalc {
    pub fn new() -> Self {
        Self
    }

    /// Returns the number of block dice given final attacker and defender strength totals.
    /// Positive result → attacker picks dice. Negative → defender picks dice.
    ///   attacker > 2× defender  →  +3
    ///   attacker >    defender  →  +2
    ///   equal                   →  +1
    ///   attacker <    defender  →  -2
    ///   attacker < 0.5× defender → -3
    pub fn block_dice_count(attacker_str: i32, defender_str: i32) -> i32 {
        if attacker_str > 2 * defender_str {
            return 3;
        }
        if attacker_str > defender_str {
            return 2;
        }
        if 2 * attacker_str < defender_str {
            return -3;
        }
        if attacker_str < defender_str {
            return -2;
        }
        1
    }

    /// Applies the "add block die" skill bonus (e.g. Horns during blitz).
    /// Only triggers when the current count is 1 or 2 (cannot exceed 3, no effect on negative).
    pub fn apply_add_block_die(dice: i32) -> i32 {
        if dice == 1 || dice == 2 {
            dice + 1
        } else {
            dice
        }
    }

    /// BB2016 multi-block: defender strength +2, attacker unchanged.
    pub fn multi_block_defender_modifier_bb2016() -> i32 {
        2
    }

    /// BB2016 multi-block: attacker strength unchanged.
    pub fn multi_block_attacker_modifier_bb2016() -> i32 {
        0
    }

    /// BB2020/BB2025 multi-block: attacker strength -2.
    pub fn multi_block_attacker_modifier_bb2020() -> i32 {
        -2
    }

    /// BB2020/BB2025 multi-block: defender strength unchanged.
    pub fn multi_block_defender_modifier_bb2020() -> i32 {
        0
    }
}

impl Default for BlockDiceCalc {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_strength_gives_one_die() {
        assert_eq!(BlockDiceCalc::block_dice_count(3, 3), 1);
    }

    #[test]
    fn attacker_strictly_more_than_double_gives_three_dice() {
        // 7 > 2*3 = 6 → true → 3 dice
        assert_eq!(BlockDiceCalc::block_dice_count(7, 3), 3);
    }

    #[test]
    fn attacker_greater_gives_two_dice() {
        assert_eq!(BlockDiceCalc::block_dice_count(4, 3), 2);
    }

    #[test]
    fn defender_greater_gives_minus_two_dice() {
        assert_eq!(BlockDiceCalc::block_dice_count(3, 4), -2);
    }

    #[test]
    fn defender_double_attacker_gives_minus_three_dice() {
        assert_eq!(BlockDiceCalc::block_dice_count(3, 7), -3);
    }

    #[test]
    fn apply_add_block_die_one_becomes_two() {
        assert_eq!(BlockDiceCalc::apply_add_block_die(1), 2);
    }

    #[test]
    fn apply_add_block_die_two_becomes_three() {
        assert_eq!(BlockDiceCalc::apply_add_block_die(2), 3);
    }

    #[test]
    fn apply_add_block_die_three_unchanged() {
        assert_eq!(BlockDiceCalc::apply_add_block_die(3), 3);
    }

    #[test]
    fn apply_add_block_die_negative_unchanged() {
        assert_eq!(BlockDiceCalc::apply_add_block_die(-2), -2);
        assert_eq!(BlockDiceCalc::apply_add_block_die(-3), -3);
    }

    #[test]
    fn exactly_double_threshold_gives_two_dice() {
        // attacker = 2 * defender (exactly double, not strictly greater than double)
        // 6 > 2*3=6 → false; 6 > 3 → true → 2 dice
        assert_eq!(BlockDiceCalc::block_dice_count(6, 3), 2);
        // attacker=5, defender=3: 5 > 6? no. 5 > 3? yes → 2
        assert_eq!(BlockDiceCalc::block_dice_count(5, 3), 2);
    }

    #[test]
    fn bb2016_multi_block_modifiers() {
        assert_eq!(BlockDiceCalc::multi_block_defender_modifier_bb2016(), 2);
        assert_eq!(BlockDiceCalc::multi_block_attacker_modifier_bb2016(), 0);
    }

    #[test]
    fn bb2020_multi_block_modifiers() {
        assert_eq!(BlockDiceCalc::multi_block_attacker_modifier_bb2020(), -2);
        assert_eq!(BlockDiceCalc::multi_block_defender_modifier_bb2020(), 0);
    }
}
