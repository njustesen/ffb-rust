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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agility_roll_base_bb2016_ag3_is_4() {
        assert_eq!(AgilityCalc::agility_roll_base_bb2016(3), 4);
    }

    #[test]
    fn agility_roll_base_bb2016_ag1_is_6() {
        assert_eq!(AgilityCalc::agility_roll_base_bb2016(1), 6);
    }

    #[test]
    fn agility_roll_base_bb2016_ag6_is_1() {
        assert_eq!(AgilityCalc::agility_roll_base_bb2016(6), 1);
    }

    #[test]
    fn agility_roll_base_bb2016_ag7_clamps_to_1() {
        assert_eq!(AgilityCalc::agility_roll_base_bb2016(7), 1);
    }

    #[test]
    fn minimum_roll_dodge_bb2016_ag3_no_mods() {
        // base=4, -1=3, floor 2 → 3
        assert_eq!(AgilityCalc::minimum_roll_dodge_bb2016(3, 0), 3);
    }

    #[test]
    fn minimum_roll_dodge_bb2016_floored_at_2() {
        // ag=6, base=1, -1+0=0 → clamped to 2
        assert_eq!(AgilityCalc::minimum_roll_dodge_bb2016(6, 0), 2);
    }

    #[test]
    fn minimum_roll_catch_bb2016_ag3_no_mods() {
        // base=4, +0=4
        assert_eq!(AgilityCalc::minimum_roll_catch_bb2016(3, 0), 4);
    }

    #[test]
    fn minimum_roll_interception_bb2016_ag3() {
        // base=4, +2=6
        assert_eq!(AgilityCalc::minimum_roll_interception_bb2016(3, 0), 6);
    }

    #[test]
    fn minimum_roll_bb2020_ag3_no_mods() {
        assert_eq!(AgilityCalc::minimum_roll_bb2020(3, 0), 3);
    }

    #[test]
    fn minimum_roll_bb2020_floored_at_2() {
        // ag=1, mod=-5 → max(2,-4)=2
        assert_eq!(AgilityCalc::minimum_roll_bb2020(1, -5), 2);
    }

    #[test]
    fn minimum_roll_dodge_dispatches_bb2016() {
        let result = AgilityCalc::minimum_roll_dodge(3, 0, Rules::Bb2016);
        assert_eq!(result, AgilityCalc::minimum_roll_dodge_bb2016(3, 0));
    }

    #[test]
    fn minimum_roll_dodge_dispatches_bb2020() {
        let result = AgilityCalc::minimum_roll_dodge(3, 0, Rules::Bb2020);
        assert_eq!(result, AgilityCalc::minimum_roll_bb2020(3, 0));
    }

    #[test]
    fn minimum_roll_dodge_dispatches_bb2025() {
        let result = AgilityCalc::minimum_roll_dodge(4, 1, Rules::Bb2025);
        assert_eq!(result, AgilityCalc::minimum_roll_bb2020(4, 1));
    }

    #[test]
    fn minimum_roll_base_bb2016_same_as_catch() {
        for ag in 1..=6 {
            assert_eq!(
                AgilityCalc::minimum_roll_base_bb2016(ag, 0),
                AgilityCalc::minimum_roll_catch_bb2016(ag, 0)
            );
        }
    }

    #[test]
    fn modifier_increases_target() {
        let base = AgilityCalc::minimum_roll_bb2020(3, 0);
        let harder = AgilityCalc::minimum_roll_bb2020(3, 2);
        assert_eq!(harder, base + 2);
    }
}
