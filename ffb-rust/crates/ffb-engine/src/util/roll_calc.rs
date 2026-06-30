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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roll_6_always_succeeds() {
        assert!(RollCalc::is_skill_roll_successful(6, 7));
    }

    #[test]
    fn roll_1_always_fails() {
        assert!(!RollCalc::is_skill_roll_successful(1, 2));
    }

    #[test]
    fn roll_meets_minimum_succeeds() {
        assert!(RollCalc::is_skill_roll_successful(4, 4));
    }

    #[test]
    fn roll_below_minimum_fails() {
        assert!(!RollCalc::is_skill_roll_successful(3, 4));
    }

    #[test]
    fn roll_above_minimum_succeeds() {
        assert!(RollCalc::is_skill_roll_successful(5, 4));
    }

    #[test]
    fn armour_bb2016_strictly_greater() {
        // roll > armour required
        assert!(!RollCalc::is_armour_broken(8, 8, Rules::Bb2016));
        assert!(RollCalc::is_armour_broken(8, 9, Rules::Bb2016));
    }

    #[test]
    fn armour_bb2020_equal_breaks() {
        assert!(RollCalc::is_armour_broken(8, 8, Rules::Bb2020));
        assert!(!RollCalc::is_armour_broken(8, 7, Rules::Bb2020));
    }

    #[test]
    fn armour_bb2025_equal_breaks() {
        assert!(RollCalc::is_armour_broken(9, 9, Rules::Bb2025));
    }

    #[test]
    fn apply_fixed_armour_reduction_bb2016_cap_7() {
        assert_eq!(RollCalc::apply_fixed_armour_reduction(9, Rules::Bb2016), 7);
        assert_eq!(RollCalc::apply_fixed_armour_reduction(6, Rules::Bb2016), 6);
    }

    #[test]
    fn apply_fixed_armour_reduction_bb2020_cap_8() {
        assert_eq!(RollCalc::apply_fixed_armour_reduction(9, Rules::Bb2020), 8);
        assert_eq!(RollCalc::apply_fixed_armour_reduction(7, Rules::Bb2020), 7);
    }

    #[test]
    fn minimum_roll_gfi_no_modifier() {
        assert_eq!(RollCalc::minimum_roll_going_for_it(0), 2);
    }

    #[test]
    fn minimum_roll_gfi_positive_modifier() {
        assert_eq!(RollCalc::minimum_roll_going_for_it(1), 3);
    }

    #[test]
    fn minimum_roll_gfi_negative_modifier_floored_at_2() {
        assert_eq!(RollCalc::minimum_roll_going_for_it(-5), 2);
    }
}
