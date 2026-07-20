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

// Test mirror of com.fumbbl.ffb.server.util.BlockDiceCalcTest
#[cfg(test)]
mod tests {
    use super::*;

    // ── Equal strength ────────────────────────────────────────────────────────

    #[test]
    fn equal_strength_returns_one_die() {
        assert_eq!(BlockDiceCalc::block_dice_count(3, 3), 1);
    }

    #[test]
    fn equal_strength_at_edge_1_vs_1_returns_one_die() {
        assert_eq!(BlockDiceCalc::block_dice_count(1, 1), 1);
    }

    // ── Attacker advantage ───────────────────────────────────────────────────

    #[test]
    fn attacker_stronger_returns_two_dice() {
        for (attacker, defender) in [(4, 3), (5, 4), (5, 3), (3, 2), (6, 4)] {
            assert_eq!(
                BlockDiceCalc::block_dice_count(attacker, defender),
                2,
                "attacker={attacker} defender={defender}"
            );
        }
    }

    #[test]
    fn attacker_more_than_double_strength_returns_three_dice() {
        for (attacker, defender) in [(7, 3), (6, 2), (4, 1), (10, 4), (8, 3)] {
            assert_eq!(
                BlockDiceCalc::block_dice_count(attacker, defender),
                3,
                "attacker={attacker} defender={defender}"
            );
        }
    }

    #[test]
    fn attacker_exactly_double_defender_returns_two_dice() {
        // 6 > 2×3 → false (strictly greater), so 3 dice requires strictly more than double
        assert_eq!(BlockDiceCalc::block_dice_count(6, 3), 2);
    }

    #[test]
    fn attacker_one_above_double_returns_three_dice() {
        assert_eq!(BlockDiceCalc::block_dice_count(7, 3), 3);
    }

    // ── Defender advantage ───────────────────────────────────────────────────

    #[test]
    fn defender_stronger_returns_minus_two_dice() {
        for (attacker, defender) in [(3, 4), (2, 3), (4, 5), (3, 5), (3, 6)] {
            assert_eq!(
                BlockDiceCalc::block_dice_count(attacker, defender),
                -2,
                "attacker={attacker} defender={defender}"
            );
        }
    }

    #[test]
    fn defender_one_above_double_returns_minus_three_dice() {
        assert_eq!(BlockDiceCalc::block_dice_count(3, 7), -3);
    }

    #[test]
    fn defender_exactly_double_attacker_returns_minus_two_dice() {
        // 2×3 < 6 → false (strictly less), so -3 requires strictly more than double
        assert_eq!(BlockDiceCalc::block_dice_count(3, 6), -2);
    }

    #[test]
    fn defender_more_than_double_attacker_returns_minus_three_dice() {
        for (attacker, defender) in [(1, 3), (2, 5), (3, 7), (4, 9)] {
            assert_eq!(
                BlockDiceCalc::block_dice_count(attacker, defender),
                -3,
                "attacker={attacker} defender={defender}"
            );
        }
    }

    // ── add_block_die bonus ──────────────────────────────────────────────────

    #[test]
    fn add_block_die_on_one_die_returns_two() {
        assert_eq!(BlockDiceCalc::apply_add_block_die(1), 2);
    }

    #[test]
    fn add_block_die_on_two_dice_returns_three() {
        assert_eq!(BlockDiceCalc::apply_add_block_die(2), 3);
    }

    #[test]
    fn add_block_die_on_three_dice_no_change() {
        assert_eq!(BlockDiceCalc::apply_add_block_die(3), 3);
    }

    #[test]
    fn add_block_die_on_negative_dice_no_change() {
        assert_eq!(BlockDiceCalc::apply_add_block_die(-2), -2);
        assert_eq!(BlockDiceCalc::apply_add_block_die(-3), -3);
    }

    // ── Multi-block modifiers ────────────────────────────────────────────────

    #[test]
    fn bb2016_multi_block_defender_gets_plus_2() {
        assert_eq!(BlockDiceCalc::multi_block_defender_modifier_bb2016(), 2);
        assert_eq!(BlockDiceCalc::multi_block_attacker_modifier_bb2016(), 0);
    }

    #[test]
    fn bb2020_multi_block_attacker_gets_minus_2() {
        assert_eq!(BlockDiceCalc::multi_block_attacker_modifier_bb2020(), -2);
        assert_eq!(BlockDiceCalc::multi_block_defender_modifier_bb2020(), 0);
    }

    #[test]
    fn multi_block_bb2016_reduces_attacker_advantage() {
        // ST4 attacker vs ST3 defender → normally 2 dice
        // BB2016 multi-block: defender +2 → ST3+2=5, attacker ST4 < ST5 → -2 dice
        let def_str = 3 + BlockDiceCalc::multi_block_defender_modifier_bb2016();
        let att_str = 4 + BlockDiceCalc::multi_block_attacker_modifier_bb2016();
        assert_eq!(BlockDiceCalc::block_dice_count(att_str, def_str), -2);
    }

    #[test]
    fn multi_block_bb2020_reduces_attacker_strength() {
        // ST4 attacker vs ST3 defender → normally 2 dice
        // BB2020 multi-block: attacker -2 → ST4-2=2 vs ST3 → -2 dice
        let def_str = 3 + BlockDiceCalc::multi_block_defender_modifier_bb2020();
        let att_str = 4 + BlockDiceCalc::multi_block_attacker_modifier_bb2020();
        assert_eq!(BlockDiceCalc::block_dice_count(att_str, def_str), -2);
    }
}
