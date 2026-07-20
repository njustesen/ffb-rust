use ffb_model::enums::Rules;
use crate::modifiers::Modifier;

fn sum_modifiers(modifiers: &[Modifier]) -> i32 {
    modifiers.iter().map(|m| m.value).sum()
}

// ── BB2016 agility formula ────────────────────────────────────────────────────

/// BB2016 base target for agility rolls: `7 - min(ag, 6)`.
///
/// AG1→6, AG2→5, AG3→4, AG4→3, AG5→2, AG6+→1.
pub fn agility_roll_base_bb2016(agility: i32) -> i32 {
    7 - agility.min(6)
}

/// Minimum roll for dodge or pickup (BB2016).
/// Uses base - 1 (dodge gets a -1 bonus over catch).
pub fn minimum_roll_dodge_bb2016(agility: i32, modifier_total: i32) -> i32 {
    (agility_roll_base_bb2016(agility) - 1 + modifier_total).max(2)
}

/// Minimum roll for catch (BB2016). Uses base directly (no -1 bonus).
pub fn minimum_roll_catch_bb2016(agility: i32, modifier_total: i32) -> i32 {
    (agility_roll_base_bb2016(agility) + modifier_total).max(2)
}

/// Minimum roll for jump-up, leap, or hypnotic gaze (BB2016). Uses base directly.
pub fn minimum_roll_base_bb2016(agility: i32, modifier_total: i32) -> i32 {
    (agility_roll_base_bb2016(agility) + modifier_total).max(2)
}

/// Minimum roll for interception (BB2016). Harder by +2.
pub fn minimum_roll_intercept_bb2016(agility: i32, modifier_total: i32) -> i32 {
    (agility_roll_base_bb2016(agility) + 2 + modifier_total).max(2)
}

// ── BB2020/BB2025 agility formula ─────────────────────────────────────────────

/// Core BB2020/BB2025 agility roll: `max(2, agility + sum_of_modifiers)`.
///
/// The agility stat IS the target number directly (e.g., "3+" → ag=3).
/// All action types share this formula.
pub fn minimum_roll(agility: i32, modifiers: &[Modifier]) -> i32 {
    (agility + sum_modifiers(modifiers)).max(2)
}

/// Minimum roll to dodge (BB2020/BB2025).
pub fn minimum_roll_dodge(agility: i32, modifiers: &[Modifier]) -> i32 {
    minimum_roll(agility, modifiers)
}

/// Minimum roll to pick up the ball (BB2020/BB2025).
pub fn minimum_roll_pickup(agility: i32, modifiers: &[Modifier]) -> i32 {
    minimum_roll(agility, modifiers)
}

/// Minimum roll to catch (BB2020/BB2025).
pub fn minimum_roll_catch(agility: i32, modifiers: &[Modifier]) -> i32 {
    minimum_roll(agility, modifiers)
}

/// Minimum roll to intercept (BB2020/BB2025).
pub fn minimum_roll_intercept(agility: i32, modifiers: &[Modifier]) -> i32 {
    minimum_roll(agility, modifiers)
}

/// Minimum roll for Right Stuff landing (BB2020/BB2025).
pub fn minimum_roll_right_stuff(agility: i32, modifiers: &[Modifier]) -> i32 {
    minimum_roll(agility, modifiers)
}

/// Minimum roll for a jump (Leap) (BB2020/BB2025).
pub fn minimum_roll_jump(agility: i32, modifiers: &[Modifier]) -> i32 {
    minimum_roll(agility, modifiers)
}

/// Minimum roll for Jump Up (BB2020/BB2025).
pub fn minimum_roll_jump_up(agility: i32, modifiers: &[Modifier]) -> i32 {
    minimum_roll(agility, modifiers)
}

/// Minimum roll for Hypnotic Gaze (uses agility or skill-specified value).
pub fn minimum_roll_hypnotic_gaze(base_value: i32, modifiers: &[Modifier]) -> i32 {
    minimum_roll(base_value, modifiers)
}

/// Go-for-it always needs a 2+ (no agility dependency).
pub fn minimum_roll_gfi(_rules: Rules) -> i32 {
    2
}

/// Safe Throw: straight agility roll, no modifiers.
pub fn minimum_roll_safe_throw(agility: i32) -> i32 {
    minimum_roll(agility, &[])
}

/// Minimum roll to stand up when required (MA ≤ 3 requires this roll).
pub fn minimum_roll_stand_up() -> i32 {
    4
}

/// Edition-dispatched catch minimum roll (integer modifier total).
///
/// BB2016: uses `agility_roll_base_bb2016(ag) + modifier_total`, floor 2.
/// BB2020/BB2025: uses `ag + modifier_total`, floor 2.
pub fn minimum_roll_catch_edition(agility: i32, modifier_total: i32, rules: Rules) -> i32 {
    match rules {
        Rules::Bb2016 => minimum_roll_catch_bb2016(agility, modifier_total),
        _ => (agility + modifier_total).max(2),
    }
}

/// Edition-dispatched interception minimum roll (integer modifier total).
///
/// BB2016: base + 2 (harder than catch).
/// BB2020/BB2025: same formula as catch (no +2 penalty).
pub fn minimum_roll_intercept_edition(agility: i32, modifier_total: i32, rules: Rules) -> i32 {
    match rules {
        Rules::Bb2016 => minimum_roll_intercept_bb2016(agility, modifier_total),
        _ => (agility + modifier_total).max(2),
    }
}

// Tests exercising the ffb-mechanics API directly; names aligned with the
// Java-derived tests in AgilityCalcTest / CatchCalcTest (mirrored 1:1 in
// ffb-engine::util::{agility_calc, catch_calc}). Kept here only where they
// cover mechanics-specific API (free functions, Modifier slices, edition
// dispatchers) that the engine mirrors do not touch.
#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;

    // ── BB2016 base ───────────────────────────────────────────────────────────

    #[test]
    fn bb2016_agility_base() {
        assert_eq!(agility_roll_base_bb2016(1), 6);
        assert_eq!(agility_roll_base_bb2016(2), 5);
        assert_eq!(agility_roll_base_bb2016(3), 4);
        assert_eq!(agility_roll_base_bb2016(4), 3);
        assert_eq!(agility_roll_base_bb2016(5), 2);
        assert_eq!(agility_roll_base_bb2016(6), 1);
        assert_eq!(agility_roll_base_bb2016(7), 1); // capped
        assert_eq!(agility_roll_base_bb2016(9), 1); // capped
    }

    #[test]
    fn bb2016_dodge_no_modifiers() {
        // AG1→5, AG2→4, AG3→3, AG4→2, AG5→2 (floor), AG6→2 (floor)
        assert_eq!(minimum_roll_dodge_bb2016(1, 0), 5); // base 6 - 1
        assert_eq!(minimum_roll_dodge_bb2016(2, 0), 4); // base 5 - 1
        assert_eq!(minimum_roll_dodge_bb2016(3, 0), 3); // base 4 - 1
        assert_eq!(minimum_roll_dodge_bb2016(4, 0), 2); // base 3 - 1
        assert_eq!(minimum_roll_dodge_bb2016(5, 0), 2); // 2 - 1 = 1 → floor 2
        assert_eq!(minimum_roll_dodge_bb2016(6, 0), 2); // 1 - 1 = 0 → floor 2
    }

    #[test]
    fn bb2016_dodge_with_positive_modifier_increases() {
        // AG4 base dodge = 2; +1 tackle zone = 3
        assert_eq!(minimum_roll_dodge_bb2016(4, 1), 3);
        // AG4 base dodge = 2; +2 tackle zones = 4
        assert_eq!(minimum_roll_dodge_bb2016(4, 2), 4);
    }

    #[test]
    fn bb2016_dodge_with_negative_modifier_decreases_but_floor_at_2() {
        // AG3 base dodge = 3; Dodge skill -1 = 2
        assert_eq!(minimum_roll_dodge_bb2016(3, -1), 2);
        // AG2 base dodge = 4; -10 modifiers → 2
        assert_eq!(minimum_roll_dodge_bb2016(2, -10), 2);
    }

    #[test]
    fn bb2016_catch_harder_than_dodge_for_same_ag() {
        for ag in 1..=4 {
            assert_eq!(
                minimum_roll_catch_bb2016(ag, 0),
                minimum_roll_dodge_bb2016(ag, 0) + 1,
                "ag={ag}"
            );
        }
    }

    // ── BB2020 (Modifier-slice API, not covered by the engine mirrors) ───────

    #[test]
    fn bb2020_direct_agility_no_modifiers() {
        for ag in 2..=6 {
            assert_eq!(minimum_roll(ag, &[]), ag, "ag={ag}");
        }
    }

    #[test]
    fn bb2020_with_negative_modifier_floor_at_2() {
        let m5 = Modifier::new("test", -5, Rules::Common);
        assert_eq!(minimum_roll(3, &[m5]), 2); // floor
        let m1 = Modifier::new("test", -1, Rules::Common);
        assert_eq!(minimum_roll(2, &[m1]), 2); // 1 → 2
    }

    #[test]
    fn dispatched_bb2016_vs_bb2020_differ_for_same_ag4() {
        // BB2016 AG4: dodge target = 2
        // BB2020 AG4 (4+ player): target = 4
        assert_eq!(minimum_roll_dodge_bb2016(4, 0), 2);
        assert_eq!(minimum_roll(4, &[]), 4);
    }

    #[test]
    fn bb2020_with_positive_modifier() {
        // AG3 + modifier 1 = 4
        let m = Modifier::new("test", 1, Rules::Common);
        assert_eq!(minimum_roll(3, &[m]), 4);
    }

    // ── Common ───────────────────────────────────────────────────────────────

    #[test]
    fn gfi_always_2() {
        assert_eq!(minimum_roll_gfi(Rules::Bb2016), 2);
        assert_eq!(minimum_roll_gfi(Rules::Bb2020), 2);
        assert_eq!(minimum_roll_gfi(Rules::Bb2025), 2);
    }

    // ── Edition-dispatched catch ──────────────────────────────────────────────

    #[test]
    fn dispatched_catch_bb2016_and_bb2020_differ_for_ag4() {
        assert_eq!(minimum_roll_catch_edition(4, 0, Rules::Bb2016), 3);
        assert_eq!(minimum_roll_catch_edition(4, 0, Rules::Bb2020), 4);
        assert_eq!(minimum_roll_catch_edition(4, 0, Rules::Bb2025), 4);
    }

    #[test]
    fn bb2020_catch_floored_at_2() {
        assert_eq!(minimum_roll_catch_edition(1, -5, Rules::Bb2020), 2);
    }

    #[test]
    fn bb2020_catch_with_modifier() {
        // AG3 + rain (+1) = 4
        assert_eq!(minimum_roll_catch_edition(3, 1, Rules::Bb2020), 4);
    }

    // ── Edition-dispatched interception ──────────────────────────────────────

    #[test]
    fn dispatched_interception_bb2016_harder_than_bb2020() {
        assert_eq!(minimum_roll_intercept_edition(4, 0, Rules::Bb2016), 5);
        assert_eq!(minimum_roll_intercept_edition(4, 0, Rules::Bb2020), 4);
        assert_eq!(minimum_roll_intercept_edition(4, 0, Rules::Bb2025), 4);
    }

    #[test]
    fn bb2020_interception_same_as_catch() {
        for ag in 2..=6 {
            assert_eq!(
                minimum_roll_catch_edition(ag, 0, Rules::Bb2020),
                minimum_roll_intercept_edition(ag, 0, Rules::Bb2020),
                "ag={ag}"
            );
        }
    }

    // ── Full catch agility table (CatchCalcTest parity) ──────────────────────

    #[test]
    fn bb2016_catch_agility_table() {
        // AG1→6, AG2→5, AG3→4, AG4→3, AG5→2, AG6→2 (floor 2)
        assert_eq!(minimum_roll_catch_bb2016(1, 0), 6);
        assert_eq!(minimum_roll_catch_bb2016(2, 0), 5);
        assert_eq!(minimum_roll_catch_bb2016(3, 0), 4);
        assert_eq!(minimum_roll_catch_bb2016(4, 0), 3);
        assert_eq!(minimum_roll_catch_bb2016(5, 0), 2);
        assert_eq!(minimum_roll_catch_bb2016(6, 0), 2);
    }

    #[test]
    fn bb2016_catch_with_positive_modifier_harder() {
        // AG3 + rain (+1) = 5
        assert_eq!(minimum_roll_catch_bb2016(3, 1), 5);
    }

    #[test]
    fn bb2016_catch_with_negative_modifier_easier() {
        // AG4 - 1 benefit: 3 - 1 = 2 (floor 2)
        assert_eq!(minimum_roll_catch_bb2016(4, -1), 2);
    }

    #[test]
    fn bb2016_catch_floored_at_2() {
        assert_eq!(minimum_roll_catch_bb2016(6, -10), 2);
    }

    #[test]
    fn bb2016_interception_agility_table() {
        // base + 2: AG1→8, AG2→7, AG3→6, AG4→5, AG5→4, AG6→3
        assert_eq!(minimum_roll_intercept_bb2016(1, 0), 8);
        assert_eq!(minimum_roll_intercept_bb2016(2, 0), 7);
        assert_eq!(minimum_roll_intercept_bb2016(3, 0), 6);
        assert_eq!(minimum_roll_intercept_bb2016(4, 0), 5);
        assert_eq!(minimum_roll_intercept_bb2016(5, 0), 4);
        assert_eq!(minimum_roll_intercept_bb2016(6, 0), 3);
    }

    #[test]
    fn bb2016_interception_harder_than_catch() {
        for ag in 1..=6 {
            let intercept = minimum_roll_intercept_bb2016(ag, 0);
            let catch_ = minimum_roll_catch_bb2016(ag, 0);
            assert!(intercept >= catch_, "ag={ag}: intercept={intercept} catch={catch_}");
        }
    }

    #[test]
    fn bb2020_catch_equals_ag() {
        // In BB2020 the agility stat IS the target directly
        for ag in 2..=6 {
            assert_eq!(minimum_roll_catch_edition(ag, 0, Rules::Bb2020), ag, "ag={ag}");
        }
    }

}
