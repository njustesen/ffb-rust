// 1:1 translation of com.fumbbl.ffb.server.util.AgilityCalc
use ffb_model::enums::Rules;

pub struct AgilityCalc;

impl AgilityCalc {
    pub fn new() -> Self {
        Self
    }

    /// BB2016 base target for agility rolls before action-specific adjustment.
    /// Agility 1 → 6, 2 → 5, 3 → 4, 4 → 3, 5 → 2, 6+ → 1.
    pub fn agility_roll_base_bb2016(agility: i32) -> i32 {
        7 - agility.min(6)
    }

    /// Minimum roll for a dodge or pickup (BB2016).
    /// Dodge and pickup share the same formula (base - 1, not base).
    pub fn minimum_roll_dodge_bb2016(agility: i32, modifier_total: i32) -> i32 {
        2_i32.max(Self::agility_roll_base_bb2016(agility) - 1 + modifier_total)
    }

    /// Minimum roll for a catch (BB2016).
    /// Catch uses the base without the -1 dodge bonus.
    pub fn minimum_roll_catch_bb2016(agility: i32, modifier_total: i32) -> i32 {
        2_i32.max(Self::agility_roll_base_bb2016(agility) + modifier_total)
    }

    /// Minimum roll for a jump-up, leap, or hypnotic gaze (BB2016).
    pub fn minimum_roll_base_bb2016(agility: i32, modifier_total: i32) -> i32 {
        2_i32.max(Self::agility_roll_base_bb2016(agility) + modifier_total)
    }

    /// Minimum roll for an interception (BB2016).
    /// Interception is harder by +2.
    pub fn minimum_roll_interception_bb2016(agility: i32, modifier_total: i32) -> i32 {
        2_i32.max(Self::agility_roll_base_bb2016(agility) + 2 + modifier_total)
    }

    /// Minimum roll for any agility-based action (BB2020/BB2025).
    /// The agility stat is the target number directly ("3+" → ag=3).
    pub fn minimum_roll_bb2020(agility: i32, modifier_total: i32) -> i32 {
        2_i32.max(agility + modifier_total)
    }

    /// Minimum roll for any agility-based action, edition-dispatched.
    /// For BB2016 uses the dodge/pickup formula (base - 1 + mods).
    /// For BB2020/BB2025 uses the direct formula (ag + mods).
    pub fn minimum_roll_dodge(agility: i32, modifier_total: i32, rules: Rules) -> i32 {
        if rules == Rules::Bb2016 {
            Self::minimum_roll_dodge_bb2016(agility, modifier_total)
        } else {
            Self::minimum_roll_bb2020(agility, modifier_total)
        }
    }
}

impl Default for AgilityCalc {
    fn default() -> Self {
        Self::new()
    }
}

// Test mirror of com.fumbbl.ffb.server.util.AgilityCalcTest
#[cfg(test)]
mod tests {
    use super::*;

    // ── agility_roll_base_bb2016 ─────────────────────────────────────────────

    #[test]
    fn bb2016_agility_base() {
        for (ag, expected) in [(1, 6), (2, 5), (3, 4), (4, 3), (5, 2), (6, 1), (7, 1), (9, 1)] {
            assert_eq!(AgilityCalc::agility_roll_base_bb2016(ag), expected, "ag={ag}");
        }
    }

    // ── minimum_roll_dodge_bb2016 ────────────────────────────────────────────

    #[test]
    fn bb2016_dodge_no_modifiers() {
        for (ag, expected) in [(1, 5), (2, 4), (3, 3), (4, 2), (5, 2), (6, 2)] {
            assert_eq!(AgilityCalc::minimum_roll_dodge_bb2016(ag, 0), expected, "ag={ag}");
        }
    }

    #[test]
    fn bb2016_dodge_with_positive_modifier_increases() {
        // AG4 base dodge = 2; +1 tackle zone = 3
        assert_eq!(AgilityCalc::minimum_roll_dodge_bb2016(4, 1), 3);
        // AG4 base dodge = 2; +2 tackle zones = 4
        assert_eq!(AgilityCalc::minimum_roll_dodge_bb2016(4, 2), 4);
    }

    #[test]
    fn bb2016_dodge_with_negative_modifier_decreases_but_floor_at_2() {
        // AG3 base dodge = 3; Dodge skill -1 = 2
        assert_eq!(AgilityCalc::minimum_roll_dodge_bb2016(3, -1), 2);
        // AG2 base dodge = 4; -10 modifiers → 2
        assert_eq!(AgilityCalc::minimum_roll_dodge_bb2016(2, -10), 2);
    }

    // ── minimum_roll_catch_bb2016 ────────────────────────────────────────────

    #[test]
    fn bb2016_catch_no_modifiers() {
        for (ag, expected) in [(1, 6), (2, 5), (3, 4), (4, 3), (5, 2), (6, 2)] {
            assert_eq!(AgilityCalc::minimum_roll_catch_bb2016(ag, 0), expected, "ag={ag}");
        }
    }

    #[test]
    fn bb2016_catch_harder_than_dodge_for_same_ag() {
        // Same AG, catch is always 1 harder than dodge (unless both at floor)
        for ag in 1..=4 {
            let dodge = AgilityCalc::minimum_roll_dodge_bb2016(ag, 0);
            let catch_ = AgilityCalc::minimum_roll_catch_bb2016(ag, 0);
            assert_eq!(catch_, dodge + 1, "ag={ag}");
        }
    }

    #[test]
    fn bb2016_catch_with_positive_modifier_harder() {
        // AG4, catch base = 3; +1 tackle zone → 4
        assert_eq!(AgilityCalc::minimum_roll_catch_bb2016(4, 1), 4);
        assert_eq!(AgilityCalc::minimum_roll_catch_bb2016(4, 2), 5);
    }

    #[test]
    fn bb2016_catch_with_negative_modifier_floor_at_2() {
        // AG4, catch base = 3; -1 skill → 2; can't go below 2
        assert_eq!(AgilityCalc::minimum_roll_catch_bb2016(4, -1), 2);
        assert_eq!(AgilityCalc::minimum_roll_catch_bb2016(4, -10), 2);
    }

    // ── minimum_roll_interception_bb2016 ─────────────────────────────────────

    #[test]
    fn bb2016_interception_no_modifiers() {
        for (ag, expected) in [(1, 8), (2, 7), (3, 6), (4, 5), (5, 4), (6, 3)] {
            assert_eq!(
                AgilityCalc::minimum_roll_interception_bb2016(ag, 0),
                expected,
                "ag={ag}"
            );
        }
    }

    #[test]
    fn bb2016_interception_harder_than_catch_by_2_for_same_ag() {
        // Interception is exactly 2 harder than catch for all AG (above floor)
        for ag in 1..=4 {
            let catch_ = AgilityCalc::minimum_roll_catch_bb2016(ag, 0);
            let intercept = AgilityCalc::minimum_roll_interception_bb2016(ag, 0);
            assert_eq!(intercept, catch_ + 2, "ag={ag}");
        }
    }

    // ── minimum_roll_base_bb2016 ─────────────────────────────────────────────

    #[test]
    fn bb2016_base_same_as_catch() {
        // Jump-up/leap/hypnotic gaze use the same formula as catch (base + mods)
        for ag in 1..=6 {
            assert_eq!(
                AgilityCalc::minimum_roll_catch_bb2016(ag, 0),
                AgilityCalc::minimum_roll_base_bb2016(ag, 0),
                "ag={ag}"
            );
        }
    }

    // ── minimum_roll_bb2020 ──────────────────────────────────────────────────

    #[test]
    fn bb2020_direct_agility_no_modifiers() {
        for (ag, expected) in [(2, 2), (3, 3), (4, 4), (5, 5), (6, 6)] {
            assert_eq!(AgilityCalc::minimum_roll_bb2020(ag, 0), expected, "ag={ag}");
        }
    }

    #[test]
    fn bb2020_with_positive_modifier() {
        assert_eq!(AgilityCalc::minimum_roll_bb2020(3, 1), 4); // 3 + 1 = 4
    }

    #[test]
    fn bb2020_with_negative_modifier_floor_at_2() {
        assert_eq!(AgilityCalc::minimum_roll_bb2020(3, -5), 2); // floor
        assert_eq!(AgilityCalc::minimum_roll_bb2020(2, -1), 2); // 1 → 2
    }

    // ── minimum_roll_dodge (edition-dispatched) ──────────────────────────────

    #[test]
    fn dispatched_bb2016_vs_bb2020_differ_for_same_ag4() {
        // BB2016 AG4: dodge target = 2
        // BB2020 AG4 (meaning "4+" player): target = 4
        assert_eq!(AgilityCalc::minimum_roll_dodge(4, 0, Rules::Bb2016), 2);
        assert_eq!(AgilityCalc::minimum_roll_dodge(4, 0, Rules::Bb2020), 4);
    }

    #[test]
    fn dispatched_bb2025_same_as_bb2020() {
        assert_eq!(
            AgilityCalc::minimum_roll_dodge(3, 0, Rules::Bb2020),
            AgilityCalc::minimum_roll_dodge(3, 0, Rules::Bb2025)
        );
    }

    #[test]
    #[allow(clippy::eq_op)] // Java test asserts the same call against itself; mirrored verbatim
    fn bb2020_catch_uses_same_formula_as_dodge() {
        // In BB2020 all agility actions (dodge, catch, intercept) use the same formula: ag + mods
        // So catch and dodge roll against the same target for a given AG and modifier sum
        assert_eq!(
            AgilityCalc::minimum_roll_bb2020(4, 0),
            AgilityCalc::minimum_roll_bb2020(4, 0)
        );
        assert_eq!(AgilityCalc::minimum_roll_bb2020(4, 0), 4); // "4+" player needs to roll 4+
        assert_eq!(AgilityCalc::minimum_roll_bb2020(4, -5), 2); // floored at 2
    }
}
