use ffb_model::enums::Rules;

/// Whether a single d6 skill roll (dodge, pickup, catch, etc.) succeeds.
///
/// Natural 6 always succeeds; natural 1 always fails; otherwise roll >= minimum_roll.
pub fn is_skill_roll_successful(roll: i32, minimum_roll: i32) -> bool {
    roll == 6 || (roll != 1 && roll >= minimum_roll)
}

/// Whether a 2d6 armour roll breaks the player's armour.
///
/// BB2016: roll_total must strictly exceed armour.
/// BB2020/BB2025: roll_total must equal or exceed armour.
pub fn is_armour_broken(armour: i32, roll_total: i32, rules: Rules) -> bool {
    match rules {
        Rules::Bb2016 => roll_total > armour,
        _ => roll_total >= armour,
    }
}

/// Applies the fixed-armour-reduction cap (Chainsaw and similar skills).
///
/// BB2016 caps at 7; BB2020/BB2025 caps at 8.
pub fn apply_fixed_armour_reduction(armour: i32, rules: Rules) -> i32 {
    let cap = match rules {
        Rules::Bb2016 => 7,
        _ => 8,
    };
    armour.min(cap)
}

/// Minimum roll required for a Going For It attempt.
///
/// Base is 2; positive modifiers increase it but cannot drop below 2.
pub fn minimum_roll_going_for_it(modifier_total: i32) -> i32 {
    (2 + modifier_total).max(2)
}

// NOTE: tests duplicating RollCalcTest.java live in the 1:1 mirror module
// (ffb-engine/src/util/roll_calc.rs); redundant copies were removed here.
// The two tests below sweep exhaustive value ranges the mirror does not cover;
// their names align with the Java-derived test names.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bb2025_same_as_bb2020() {
        for av in 4..=11 {
            for roll in 2..=12 {
                assert_eq!(
                    is_armour_broken(av, roll, Rules::Bb2020),
                    is_armour_broken(av, roll, Rules::Bb2025),
                    "av={av} roll={roll}"
                );
            }
        }
    }

    #[test]
    fn chainsaw_bb2025_same_as_bb2020() {
        for av in 4..=12 {
            assert_eq!(
                apply_fixed_armour_reduction(av, Rules::Bb2020),
                apply_fixed_armour_reduction(av, Rules::Bb2025),
                "av={av}"
            );
        }
    }
}
