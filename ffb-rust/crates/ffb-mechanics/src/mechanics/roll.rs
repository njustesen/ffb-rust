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

#[cfg(test)]
mod tests {
    use super::*;

    // ── is_skill_roll_successful ──────────────────────────────────────────────

    #[test]
    fn natural_six_always_succeeds() {
        assert!(is_skill_roll_successful(6, 7)); // impossible target
        assert!(is_skill_roll_successful(6, 6));
        assert!(is_skill_roll_successful(6, 2));
    }

    #[test]
    fn natural_one_always_fails() {
        assert!(!is_skill_roll_successful(1, 1));
        assert!(!is_skill_roll_successful(1, 2));
        assert!(!is_skill_roll_successful(1, 6));
    }

    #[test]
    fn normal_roll_meets_minimum() {
        assert!(is_skill_roll_successful(2, 2));
        assert!(is_skill_roll_successful(5, 4));
        assert!(!is_skill_roll_successful(4, 5));
        assert!(!is_skill_roll_successful(2, 3));
    }

    // ── is_armour_broken ─────────────────────────────────────────────────────

    #[test]
    fn bb2016_equal_roll_does_not_break() {
        assert!(!is_armour_broken(7, 7, Rules::Bb2016));
        assert!(!is_armour_broken(9, 9, Rules::Bb2016));
    }

    #[test]
    fn bb2016_over_breaks() {
        assert!(is_armour_broken(7, 8, Rules::Bb2016));
        assert!(is_armour_broken(9, 10, Rules::Bb2016));
    }

    #[test]
    fn bb2020_equal_breaks() {
        assert!(is_armour_broken(7, 7, Rules::Bb2020));
        assert!(is_armour_broken(9, 9, Rules::Bb2020));
    }

    #[test]
    fn bb2020_under_does_not_break() {
        assert!(!is_armour_broken(7, 6, Rules::Bb2020));
        assert!(!is_armour_broken(9, 8, Rules::Bb2020));
    }

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
    fn edition_boundary_at_equal_roll() {
        assert!(!is_armour_broken(8, 8, Rules::Bb2016)); // not broken
        assert!(is_armour_broken(8, 8, Rules::Bb2020));  // broken
    }

    // ── apply_fixed_armour_reduction ─────────────────────────────────────────

    #[test]
    fn bb2016_caps_at_7() {
        assert_eq!(apply_fixed_armour_reduction(10, Rules::Bb2016), 7);
        assert_eq!(apply_fixed_armour_reduction(8, Rules::Bb2016), 7);
        assert_eq!(apply_fixed_armour_reduction(7, Rules::Bb2016), 7);
        assert_eq!(apply_fixed_armour_reduction(6, Rules::Bb2016), 6); // below cap
    }

    #[test]
    fn bb2020_caps_at_8() {
        assert_eq!(apply_fixed_armour_reduction(10, Rules::Bb2020), 8);
        assert_eq!(apply_fixed_armour_reduction(9, Rules::Bb2020), 8);
        assert_eq!(apply_fixed_armour_reduction(8, Rules::Bb2020), 8);
        assert_eq!(apply_fixed_armour_reduction(7, Rules::Bb2020), 7); // below cap
    }

    #[test]
    fn bb2025_fixed_armour_same_as_bb2020() {
        for av in 4..=12 {
            assert_eq!(
                apply_fixed_armour_reduction(av, Rules::Bb2020),
                apply_fixed_armour_reduction(av, Rules::Bb2025),
                "av={av}"
            );
        }
    }

    // ── minimum_roll_going_for_it ─────────────────────────────────────────────

    #[test]
    fn no_modifier_requires_2() {
        assert_eq!(minimum_roll_going_for_it(0), 2);
    }

    #[test]
    fn positive_modifier_increases() {
        assert_eq!(minimum_roll_going_for_it(1), 3);
        assert_eq!(minimum_roll_going_for_it(2), 4);
    }

    #[test]
    fn negative_modifier_floors_at_2() {
        assert_eq!(minimum_roll_going_for_it(-1), 2);
        assert_eq!(minimum_roll_going_for_it(-5), 2);
    }

    #[test]
    fn gfi_minimum_never_below_2() {
        for mod_ in -10..=0 {
            assert_eq!(minimum_roll_going_for_it(mod_), 2, "modifier={mod_}");
        }
    }
}
