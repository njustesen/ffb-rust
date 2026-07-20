// 1:1 translation of com.fumbbl.ffb.server.util.CatchCalc
use ffb_model::enums::Rules;
use crate::util::agility_calc::AgilityCalc;

pub struct CatchCalc;

impl CatchCalc {
    pub fn new() -> Self {
        Self
    }

    /// Minimum roll for a catch (BB2016).
    /// Uses agility roll base without the -1 dodge adjustment.
    pub fn minimum_roll_catch_bb2016(agility: i32, modifier_total: i32) -> i32 {
        2_i32.max(AgilityCalc::agility_roll_base_bb2016(agility) + modifier_total)
    }

    /// Minimum roll for an interception (BB2016).
    /// Interception is harder by +2 compared to a catch.
    pub fn minimum_roll_interception_bb2016(agility: i32, modifier_total: i32) -> i32 {
        2_i32.max(AgilityCalc::agility_roll_base_bb2016(agility) + 2 + modifier_total)
    }

    /// Minimum roll for a catch in BB2020/BB2025.
    pub fn minimum_roll_catch_bb2020(agility: i32, modifier_total: i32) -> i32 {
        2_i32.max(agility + modifier_total)
    }

    /// Minimum roll for an interception in BB2020/BB2025.
    /// Same formula as catch (unlike BB2016 where interception has +2 penalty).
    pub fn minimum_roll_interception_bb2020(agility: i32, modifier_total: i32) -> i32 {
        2_i32.max(agility + modifier_total)
    }

    /// Minimum roll for a catch, edition-dispatched.
    pub fn minimum_roll_catch(agility: i32, modifier_total: i32, rules: Rules) -> i32 {
        if rules == Rules::Bb2016 {
            Self::minimum_roll_catch_bb2016(agility, modifier_total)
        } else {
            Self::minimum_roll_catch_bb2020(agility, modifier_total)
        }
    }

    /// Minimum roll for an interception, edition-dispatched.
    pub fn minimum_roll_interception(agility: i32, modifier_total: i32, rules: Rules) -> i32 {
        if rules == Rules::Bb2016 {
            Self::minimum_roll_interception_bb2016(agility, modifier_total)
        } else {
            Self::minimum_roll_interception_bb2020(agility, modifier_total)
        }
    }
}

impl Default for CatchCalc {
    fn default() -> Self {
        Self::new()
    }
}

// Test mirror of com.fumbbl.ffb.server.util.CatchCalcTest
#[cfg(test)]
mod tests {
    use super::*;

    // ── minimum_roll_catch_bb2016 ────────────────────────────────────────────

    #[test]
    fn bb2016_catch_agility_table() {
        for (ag, expected) in [(1, 6), (2, 5), (3, 4), (4, 3), (5, 2), (6, 2)] {
            assert_eq!(CatchCalc::minimum_roll_catch_bb2016(ag, 0), expected, "ag={ag}");
        }
    }

    #[test]
    fn bb2016_catch_harder_than_dodge_for_ag4() {
        // Catch uses base; dodge uses base-1
        let catch_target = CatchCalc::minimum_roll_catch_bb2016(4, 0);
        let dodge_target = AgilityCalc::minimum_roll_dodge_bb2016(4, 0);
        assert_eq!(
            catch_target,
            dodge_target + 1,
            "catch should be 1 harder than dodge for AG4"
        );
    }

    #[test]
    fn bb2016_catch_with_positive_modifier_harder() {
        // AG3, +1 rain: 4 + 1 = 5
        assert_eq!(CatchCalc::minimum_roll_catch_bb2016(3, 1), 5);
    }

    #[test]
    fn bb2016_catch_with_negative_modifier_easier() {
        // AG4, -1 sure hands bonus: 3 - 1 = 2 (floor 2)
        assert_eq!(CatchCalc::minimum_roll_catch_bb2016(4, -1), 2);
    }

    #[test]
    fn bb2016_catch_floored_at_2() {
        assert_eq!(CatchCalc::minimum_roll_catch_bb2016(6, -10), 2);
    }

    // ── minimum_roll_interception_bb2016 ─────────────────────────────────────

    #[test]
    fn bb2016_interception_agility_table() {
        for (ag, expected) in [(1, 8), (2, 7), (3, 6), (4, 5), (5, 4), (6, 3)] {
            assert_eq!(
                CatchCalc::minimum_roll_interception_bb2016(ag, 0),
                expected,
                "ag={ag}"
            );
        }
    }

    #[test]
    fn bb2016_interception_harder_than_catch() {
        for ag in 1..=6 {
            let intercept = CatchCalc::minimum_roll_interception_bb2016(ag, 0);
            let catch_ = CatchCalc::minimum_roll_catch_bb2016(ag, 0);
            assert!(
                intercept >= catch_,
                "interception should be >= catch for AG{ag}"
            );
        }
    }

    // ── minimum_roll_catch_bb2020 ────────────────────────────────────────────

    #[test]
    fn bb2020_catch_equals_ag() {
        for (ag, expected) in [(2, 2), (3, 3), (4, 4), (5, 5), (6, 6)] {
            assert_eq!(CatchCalc::minimum_roll_catch_bb2020(ag, 0), expected, "ag={ag}");
        }
    }

    #[test]
    fn bb2020_catch_floored_at_2() {
        assert_eq!(CatchCalc::minimum_roll_catch_bb2020(1, -5), 2);
    }

    #[test]
    fn bb2020_catch_with_modifier() {
        // AG3 + rain (+1) = 4
        assert_eq!(CatchCalc::minimum_roll_catch_bb2020(3, 1), 4);
    }

    // ── minimum_roll_interception_bb2020 ─────────────────────────────────────

    #[test]
    fn bb2020_interception_same_as_catch() {
        // In BB2020 interception has no +2 penalty (unlike BB2016)
        for ag in 2..=6 {
            assert_eq!(
                CatchCalc::minimum_roll_catch_bb2020(ag, 0),
                CatchCalc::minimum_roll_interception_bb2020(ag, 0),
                "BB2020 interception should equal catch for AG{ag}"
            );
        }
    }

    // ── Dispatched methods ───────────────────────────────────────────────────

    #[test]
    fn dispatched_catch_bb2016_and_bb2020_differ_for_ag4() {
        // BB2016 AG4: catch = 3; BB2020 AG4: catch = 4
        assert_eq!(CatchCalc::minimum_roll_catch(4, 0, Rules::Bb2016), 3);
        assert_eq!(CatchCalc::minimum_roll_catch(4, 0, Rules::Bb2020), 4);
        assert_eq!(CatchCalc::minimum_roll_catch(4, 0, Rules::Bb2025), 4);
    }

    #[test]
    fn dispatched_interception_bb2016_harder_than_bb2020() {
        // BB2016 AG4: interception = 5; BB2020 AG4: interception = 4
        assert_eq!(CatchCalc::minimum_roll_interception(4, 0, Rules::Bb2016), 5);
        assert_eq!(CatchCalc::minimum_roll_interception(4, 0, Rules::Bb2020), 4);
        assert_eq!(CatchCalc::minimum_roll_interception(4, 0, Rules::Bb2025), 4);
    }
}
