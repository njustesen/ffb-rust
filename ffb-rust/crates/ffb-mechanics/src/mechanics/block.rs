use ffb_model::enums::{BlockResult, Rules};

/// Map a block die roll (1–6) to a BlockResult.
/// Mirrors Java BlockResultFactory.forRoll(): 1=Skull, 2=BothDown, 3/4=Pushback, 5=PowPushback, 6=Pow.
pub fn block_result_for_roll(roll: i32) -> BlockResult {
    match roll {
        1 => BlockResult::Skull,
        2 => BlockResult::BothDown,
        5 => BlockResult::PowPushback,
        6 => BlockResult::Pow,
        _ => BlockResult::Pushback,
    }
}

/// Block dice count from pre-computed attacker and defender strength totals.
///
/// Positive → attacker picks dice. Negative → defender picks dice.
/// Mirrors ServerUtilBlock.findNrOfBlockDice() comparison logic exactly.
pub fn block_dice_count(attacker_str: i32, defender_str: i32) -> i32 {
    if attacker_str > 2 * defender_str { return 3; }
    if attacker_str > defender_str { return 2; }
    if 2 * attacker_str < defender_str { return -3; }
    if attacker_str < defender_str { return -2; }
    1
}

/// Apply the "add block die" bonus (e.g. Horns during a blitz).
/// Only triggers when the current count is 1 or 2.
pub fn apply_add_block_die(dice: i32) -> i32 {
    if dice == 1 || dice == 2 { dice + 1 } else { dice }
}

/// Strength modifier applied to the attacker during a multi-block action.
pub fn multi_block_attacker_modifier(rules: Rules) -> i32 {
    match rules {
        Rules::Bb2016 => 0,
        Rules::Bb2020 | Rules::Bb2025 | Rules::Common => -2,
    }
}

/// Strength modifier applied to the defender during a multi-block action.
pub fn multi_block_defender_modifier(rules: Rules) -> i32 {
    match rules {
        Rules::Bb2016 => 2,
        Rules::Bb2020 | Rules::Bb2025 | Rules::Common => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;

    // ── Equal strength ────────────────────────────────────────────────────────

    // ── block_result_for_roll ─────────────────────────────────────────────────

    #[test]
    fn roll_1_is_skull() {
        assert_eq!(block_result_for_roll(1), BlockResult::Skull);
    }

    #[test]
    fn roll_2_is_both_down() {
        assert_eq!(block_result_for_roll(2), BlockResult::BothDown);
    }

    #[test]
    fn roll_3_and_4_are_pushback() {
        assert_eq!(block_result_for_roll(3), BlockResult::Pushback);
        assert_eq!(block_result_for_roll(4), BlockResult::Pushback);
    }

    #[test]
    fn roll_5_is_pow_pushback() {
        assert_eq!(block_result_for_roll(5), BlockResult::PowPushback);
    }

    #[test]
    fn roll_6_is_pow() {
        assert_eq!(block_result_for_roll(6), BlockResult::Pow);
    }

    #[test]
    fn all_six_faces_covered() {
        let results: Vec<BlockResult> = (1..=6).map(block_result_for_roll).collect();
        assert_eq!(results[0], BlockResult::Skull);
        assert_eq!(results[1], BlockResult::BothDown);
        assert_eq!(results[2], BlockResult::Pushback);
        assert_eq!(results[3], BlockResult::Pushback);
        assert_eq!(results[4], BlockResult::PowPushback);
        assert_eq!(results[5], BlockResult::Pow);
    }

    // ── block_dice_count ──────────────────────────────────────────────────────

    #[test]
    fn equal_strength_returns_one_die() {
        assert_eq!(block_dice_count(3, 3), 1);
        assert_eq!(block_dice_count(1, 1), 1);
        assert_eq!(block_dice_count(5, 5), 1);
    }

    // ── Attacker advantage ────────────────────────────────────────────────────

    #[test]
    fn attacker_one_above_defender_returns_two_dice() {
        assert_eq!(block_dice_count(4, 3), 2);
        assert_eq!(block_dice_count(5, 4), 2);
        assert_eq!(block_dice_count(3, 2), 2);
    }

    #[test]
    fn attacker_exactly_double_defender_returns_two_dice() {
        // strictly greater than double is required for 3 dice
        assert_eq!(block_dice_count(6, 3), 2);
        assert_eq!(block_dice_count(4, 2), 2);
    }

    #[test]
    fn attacker_more_than_double_returns_three_dice() {
        assert_eq!(block_dice_count(7, 3), 3);
        assert_eq!(block_dice_count(5, 2), 3);
        assert_eq!(block_dice_count(4, 1), 3);
    }

    // ── Defender advantage ────────────────────────────────────────────────────

    #[test]
    fn defender_one_above_attacker_returns_minus_two() {
        assert_eq!(block_dice_count(3, 4), -2);
        assert_eq!(block_dice_count(2, 3), -2);
        assert_eq!(block_dice_count(3, 5), -2);
    }

    #[test]
    fn defender_exactly_double_attacker_returns_minus_two() {
        // strictly less than half required for -3
        assert_eq!(block_dice_count(3, 6), -2);
        assert_eq!(block_dice_count(2, 4), -2);
    }

    #[test]
    fn defender_more_than_double_attacker_returns_minus_three() {
        assert_eq!(block_dice_count(3, 7), -3);
        assert_eq!(block_dice_count(1, 3), -3);
        assert_eq!(block_dice_count(2, 5), -3);
    }

    // ── add_block_die bonus ───────────────────────────────────────────────────

    #[test]
    fn add_block_die_on_one_returns_two() {
        assert_eq!(apply_add_block_die(1), 2);
    }

    #[test]
    fn add_block_die_on_two_returns_three() {
        assert_eq!(apply_add_block_die(2), 3);
    }

    #[test]
    fn add_block_die_no_effect_on_three_or_negative() {
        assert_eq!(apply_add_block_die(3), 3);
        assert_eq!(apply_add_block_die(-2), -2);
        assert_eq!(apply_add_block_die(-3), -3);
    }

    // ── Multi-block modifiers ─────────────────────────────────────────────────

    #[test]
    fn multi_block_bb2016_defender_plus2_attacker_zero() {
        assert_eq!(multi_block_defender_modifier(Rules::Bb2016), 2);
        assert_eq!(multi_block_attacker_modifier(Rules::Bb2016), 0);
    }

    #[test]
    fn multi_block_bb2020_attacker_minus2_defender_zero() {
        assert_eq!(multi_block_attacker_modifier(Rules::Bb2020), -2);
        assert_eq!(multi_block_defender_modifier(Rules::Bb2020), 0);
        // BB2025 same as BB2020
        assert_eq!(multi_block_attacker_modifier(Rules::Bb2025), -2);
        assert_eq!(multi_block_defender_modifier(Rules::Bb2025), 0);
    }

    #[test]
    fn multi_block_bb2016_reduces_attacker_advantage() {
        // ST4 attacker vs ST3 defender → normally 2 dice
        // BB2016: defender +2 → 3+2=5, attacker stays 4 → attacker < defender → -2
        let att = 4 + multi_block_attacker_modifier(Rules::Bb2016);
        let def = 3 + multi_block_defender_modifier(Rules::Bb2016);
        assert_eq!(block_dice_count(att, def), -2);
    }

    #[test]
    fn multi_block_bb2020_reduces_attacker_strength() {
        // ST4 attacker vs ST3 defender → normally 2 dice
        // BB2020: attacker -2 → 4-2=2 vs 3 → attacker < defender → -2
        let att = 4 + multi_block_attacker_modifier(Rules::Bb2020);
        let def = 3 + multi_block_defender_modifier(Rules::Bb2020);
        assert_eq!(block_dice_count(att, def), -2);
    }
}
