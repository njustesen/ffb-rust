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

// Tests exercising the ffb-mechanics API directly; names aligned with the
// Java-derived tests in BlockDiceCalcTest / BlockResultCalcTest (mirrored 1:1
// in ffb-engine::util::{block_dice_calc, block_result_calc}). Kept because the
// mechanics free functions and the Rules-dispatched multi-block modifiers are
// separate production code not covered by the engine mirrors.
#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;

    // ── block_result_for_roll ─────────────────────────────────────────────────

    #[test]
    fn skull_on_1() {
        assert_eq!(block_result_for_roll(1), BlockResult::Skull);
    }

    #[test]
    fn both_down_on_2() {
        assert_eq!(block_result_for_roll(2), BlockResult::BothDown);
    }

    #[test]
    fn pushback_on_3() {
        assert_eq!(block_result_for_roll(3), BlockResult::Pushback);
    }

    #[test]
    fn pushback_on_4() {
        assert_eq!(block_result_for_roll(4), BlockResult::Pushback);
    }

    #[test]
    fn pow_pushback_on_5() {
        assert_eq!(block_result_for_roll(5), BlockResult::PowPushback);
    }

    #[test]
    fn pow_on_6() {
        assert_eq!(block_result_for_roll(6), BlockResult::Pow);
    }

    // ── block_dice_count ──────────────────────────────────────────────────────

    #[test]
    fn equal_strength_returns_one_die() {
        assert_eq!(block_dice_count(3, 3), 1);
    }

    #[test]
    fn equal_strength_at_edge_1_vs_1_returns_one_die() {
        assert_eq!(block_dice_count(1, 1), 1);
    }

    // ── Attacker advantage ────────────────────────────────────────────────────

    #[test]
    fn attacker_stronger_returns_two_dice() {
        for (attacker, defender) in [(4, 3), (5, 4), (5, 3), (3, 2), (6, 4)] {
            assert_eq!(
                block_dice_count(attacker, defender),
                2,
                "attacker={attacker} defender={defender}"
            );
        }
    }

    #[test]
    fn attacker_exactly_double_defender_returns_two_dice() {
        // strictly greater than double is required for 3 dice
        assert_eq!(block_dice_count(6, 3), 2);
    }

    #[test]
    fn attacker_more_than_double_strength_returns_three_dice() {
        for (attacker, defender) in [(7, 3), (6, 2), (4, 1), (10, 4), (8, 3)] {
            assert_eq!(
                block_dice_count(attacker, defender),
                3,
                "attacker={attacker} defender={defender}"
            );
        }
    }

    // ── Defender advantage ────────────────────────────────────────────────────

    #[test]
    fn defender_stronger_returns_minus_two_dice() {
        for (attacker, defender) in [(3, 4), (2, 3), (4, 5), (3, 5), (3, 6)] {
            assert_eq!(
                block_dice_count(attacker, defender),
                -2,
                "attacker={attacker} defender={defender}"
            );
        }
    }

    #[test]
    fn defender_exactly_double_attacker_returns_minus_two_dice() {
        // strictly more than double required for -3
        assert_eq!(block_dice_count(3, 6), -2);
    }

    #[test]
    fn defender_more_than_double_attacker_returns_minus_three_dice() {
        for (attacker, defender) in [(1, 3), (2, 5), (3, 7), (4, 9)] {
            assert_eq!(
                block_dice_count(attacker, defender),
                -3,
                "attacker={attacker} defender={defender}"
            );
        }
    }

    // ── add_block_die bonus ───────────────────────────────────────────────────

    #[test]
    fn add_block_die_on_one_die_returns_two() {
        assert_eq!(apply_add_block_die(1), 2);
    }

    #[test]
    fn add_block_die_on_two_dice_returns_three() {
        assert_eq!(apply_add_block_die(2), 3);
    }

    #[test]
    fn add_block_die_on_three_dice_no_change() {
        assert_eq!(apply_add_block_die(3), 3);
    }

    #[test]
    fn add_block_die_on_negative_dice_no_change() {
        assert_eq!(apply_add_block_die(-2), -2);
        assert_eq!(apply_add_block_die(-3), -3);
    }

    // ── Multi-block modifiers (Rules-dispatched mechanics API) ────────────────

    #[test]
    fn bb2016_multi_block_defender_gets_plus_2() {
        assert_eq!(multi_block_defender_modifier(Rules::Bb2016), 2);
        assert_eq!(multi_block_attacker_modifier(Rules::Bb2016), 0);
    }

    #[test]
    fn bb2020_multi_block_attacker_gets_minus_2() {
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
