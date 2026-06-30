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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimum_roll_catch_bb2016_ag3_no_mods() {
        // base = 7-3 = 4; +0 = 4
        assert_eq!(CatchCalc::minimum_roll_catch_bb2016(3, 0), 4);
    }

    #[test]
    fn minimum_roll_catch_bb2016_floored_at_2() {
        // ag=6, base=1, +0 → 1 → clamped to 2
        assert_eq!(CatchCalc::minimum_roll_catch_bb2016(6, 0), 2);
    }

    #[test]
    fn minimum_roll_interception_bb2016_ag3() {
        // base=4, +2=6
        assert_eq!(CatchCalc::minimum_roll_interception_bb2016(3, 0), 6);
    }

    #[test]
    fn minimum_roll_interception_bb2016_harder_than_catch() {
        for ag in 1..=6 {
            let catch = CatchCalc::minimum_roll_catch_bb2016(ag, 0);
            let interception = CatchCalc::minimum_roll_interception_bb2016(ag, 0);
            assert!(interception >= catch, "interception should be >= catch for ag={}", ag);
        }
    }

    #[test]
    fn minimum_roll_catch_bb2020_ag3_no_mods() {
        assert_eq!(CatchCalc::minimum_roll_catch_bb2020(3, 0), 3);
    }

    #[test]
    fn minimum_roll_interception_bb2020_same_as_catch() {
        // In BB2020, interception uses same formula as catch
        for ag in 2..=6 {
            assert_eq!(
                CatchCalc::minimum_roll_catch_bb2020(ag, 0),
                CatchCalc::minimum_roll_interception_bb2020(ag, 0)
            );
        }
    }

    #[test]
    fn minimum_roll_catch_dispatches_bb2016() {
        let result = CatchCalc::minimum_roll_catch(3, 0, Rules::Bb2016);
        assert_eq!(result, CatchCalc::minimum_roll_catch_bb2016(3, 0));
    }

    #[test]
    fn minimum_roll_catch_dispatches_bb2020() {
        let result = CatchCalc::minimum_roll_catch(3, 0, Rules::Bb2020);
        assert_eq!(result, CatchCalc::minimum_roll_catch_bb2020(3, 0));
    }

    #[test]
    fn minimum_roll_interception_dispatches_bb2016() {
        let result = CatchCalc::minimum_roll_interception(3, 0, Rules::Bb2016);
        assert_eq!(result, CatchCalc::minimum_roll_interception_bb2016(3, 0));
    }

    #[test]
    fn minimum_roll_interception_dispatches_bb2025() {
        let result = CatchCalc::minimum_roll_interception(3, 0, Rules::Bb2025);
        assert_eq!(result, CatchCalc::minimum_roll_interception_bb2020(3, 0));
    }

    #[test]
    fn modifier_makes_catch_harder() {
        let easy = CatchCalc::minimum_roll_catch_bb2020(3, 0);
        let hard = CatchCalc::minimum_roll_catch_bb2020(3, 2);
        assert_eq!(hard, easy + 2);
    }
}
