// 1:1 translation of com.fumbbl.ffb.server.util.RollCalc
use ffb_model::enums::Rules;

pub struct RollCalc;

impl RollCalc {
    pub fn new() -> Self {
        Self
    }

    /// Whether a single d6 skill roll (dodge, pickup, catch, etc.) succeeds.
    /// Blood Bowl rule: a natural 6 always succeeds regardless of minimum roll;
    /// a natural 1 always fails; otherwise the roll must meet or beat the minimum.
    pub fn is_skill_roll_successful(roll: i32, minimum_roll: i32) -> bool {
        (roll == 6) || ((roll != 1) && (roll >= minimum_roll))
    }

    /// Whether a 2d6 armour roll breaks the player's armour.
    /// BB2016: rollTotal must strictly exceed armour (roll > armour).
    /// BB2020/BB2025: rollTotal must equal or exceed armour (roll >= armour).
    pub fn is_armour_broken(armour: i32, roll_total: i32, rules: Rules) -> bool {
        if rules == Rules::Bb2016 {
            roll_total > armour
        } else {
            roll_total >= armour
        }
    }

    /// Applies the Chainsaw (or similar) fixed-armour-reduction effect.
    /// BB2016 cap is 7; BB2020/BB2025 cap is 8.
    pub fn apply_fixed_armour_reduction(armour: i32, rules: Rules) -> i32 {
        let cap = if rules == Rules::Bb2016 { 7 } else { 8 };
        armour.min(cap)
    }

    /// Minimum roll required for a Going For It attempt.
    /// Base is always 2; positive modifiers increase it but it is capped at a minimum of 2.
    pub fn minimum_roll_going_for_it(modifier_total: i32) -> i32 {
        2_i32.max(2 + modifier_total)
    }
}

impl Default for RollCalc {
    fn default() -> Self {
        Self::new()
    }
}

// Tests mirror ffb-server/src/test/java/com/fumbbl/ffb/server/util/RollCalcTest.java 1:1.
// Java @ParameterizedTest/@CsvSource methods become single Rust fns looping the same rows.
#[cfg(test)]
mod tests {
    use super::*;

    // ── is_skill_roll_successful ─────────────────────────────────────────────

    #[test]
    fn natural_six_always_succeeds_regardless_of_minimum() {
        assert!(RollCalc::is_skill_roll_successful(6, 7)); // even with impossible target
        assert!(RollCalc::is_skill_roll_successful(6, 6));
        assert!(RollCalc::is_skill_roll_successful(6, 2));
    }

    #[test]
    fn natural_one_always_fails_regardless_of_minimum() {
        assert!(!RollCalc::is_skill_roll_successful(1, 1)); // even with minimum 1
        assert!(!RollCalc::is_skill_roll_successful(1, 2));
        assert!(!RollCalc::is_skill_roll_successful(1, 6));
    }

    #[test]
    fn normal_roll_meets_or_beats_minimum() {
        for (roll, min, expected) in [
            (2, 2, true),
            (3, 2, true),
            (5, 4, true),
            (4, 5, false),
            (3, 4, false),
            (2, 3, false),
        ] {
            assert_eq!(
                RollCalc::is_skill_roll_successful(roll, min),
                expected,
                "roll={roll} min={min}"
            );
        }
    }

    // ── is_armour_broken ─────────────────────────────────────────────────────

    #[test]
    fn bb2016_armour_broken_strictly_greater() {
        for (armour, roll_total, expected) in [
            (7, 7, false), // equal: NOT broken in BB2016 (strict >)
            (7, 8, true),  // one over: broken
            (9, 9, false), // equal: not broken
            (9, 10, true), // one over: broken
            (7, 6, false), // under: not broken
        ] {
            assert_eq!(
                RollCalc::is_armour_broken(armour, roll_total, Rules::Bb2016),
                expected,
                "armour={armour} rollTotal={roll_total}"
            );
        }
    }

    #[test]
    fn bb2020_armour_broken_equal_or_greater() {
        for (armour, roll_total, expected) in [
            (7, 7, true),  // equal: broken in BB2020 (>=)
            (7, 8, true),  // one over: broken
            (9, 9, true),  // equal: broken
            (7, 6, false), // under: not broken
            (9, 8, false), // under: not broken
        ] {
            assert_eq!(
                RollCalc::is_armour_broken(armour, roll_total, Rules::Bb2020),
                expected,
                "armour={armour} rollTotal={roll_total}"
            );
        }
    }

    #[test]
    fn bb2025_same_as_bb2020() {
        // BB2025 uses the same >= comparison as BB2020
        assert_eq!(
            RollCalc::is_armour_broken(7, 7, Rules::Bb2020),
            RollCalc::is_armour_broken(7, 7, Rules::Bb2025)
        );
        assert_eq!(
            RollCalc::is_armour_broken(9, 9, Rules::Bb2020),
            RollCalc::is_armour_broken(9, 9, Rules::Bb2025)
        );
    }

    #[test]
    fn edition_difference_at_equal_roll_boundary() {
        // Exact boundary where editions differ: armour == rollTotal
        assert!(!RollCalc::is_armour_broken(8, 8, Rules::Bb2016)); // NOT broken
        assert!(RollCalc::is_armour_broken(8, 8, Rules::Bb2020)); // broken
    }

    // ── apply_fixed_armour_reduction ─────────────────────────────────────────

    #[test]
    fn chainsaw_bb2016_caps_at7() {
        assert_eq!(RollCalc::apply_fixed_armour_reduction(10, Rules::Bb2016), 7);
        assert_eq!(RollCalc::apply_fixed_armour_reduction(8, Rules::Bb2016), 7);
        assert_eq!(RollCalc::apply_fixed_armour_reduction(7, Rules::Bb2016), 7);
        assert_eq!(RollCalc::apply_fixed_armour_reduction(6, Rules::Bb2016), 6); // below cap, unchanged
    }

    #[test]
    fn chainsaw_bb2020_caps_at8() {
        assert_eq!(RollCalc::apply_fixed_armour_reduction(10, Rules::Bb2020), 8);
        assert_eq!(RollCalc::apply_fixed_armour_reduction(9, Rules::Bb2020), 8);
        assert_eq!(RollCalc::apply_fixed_armour_reduction(8, Rules::Bb2020), 8);
        assert_eq!(RollCalc::apply_fixed_armour_reduction(7, Rules::Bb2020), 7); // below cap, unchanged
    }

    #[test]
    fn chainsaw_bb2025_same_as_bb2020() {
        assert_eq!(
            RollCalc::apply_fixed_armour_reduction(10, Rules::Bb2020),
            RollCalc::apply_fixed_armour_reduction(10, Rules::Bb2025)
        );
    }

    // ── minimum_roll_going_for_it ────────────────────────────────────────────

    #[test]
    fn gfi_no_modifier_requires2() {
        assert_eq!(RollCalc::minimum_roll_going_for_it(0), 2);
    }

    #[test]
    fn gfi_positive_modifier_increases() {
        assert_eq!(RollCalc::minimum_roll_going_for_it(1), 3);
        assert_eq!(RollCalc::minimum_roll_going_for_it(2), 4);
    }

    #[test]
    fn gfi_negative_modifier_floor_at2() {
        // Negative modifiers cannot push below 2
        assert_eq!(RollCalc::minimum_roll_going_for_it(-1), 2);
        assert_eq!(RollCalc::minimum_roll_going_for_it(-5), 2);
    }

    #[test]
    fn gfi_minimum_is_always_2() {
        for modifier in -10..=0 {
            assert_eq!(
                RollCalc::minimum_roll_going_for_it(modifier),
                2,
                "modifier={modifier} should never push GFI below 2"
            );
        }
    }
}
