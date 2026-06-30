// 1:1 translation of com.fumbbl.ffb.server.util.PassCalc
use ffb_model::enums::PassingDistance;

pub struct PassCalc;

impl PassCalc {
    pub fn new() -> Self {
        Self
    }

    /// Minimum roll for a pass in BB2016.
    /// target = max(max(agBased, fumbleBoundary), 2)
    /// where agBased = 7 - min(ag,6) - distMod + modifierTotal
    ///       fumbleBoundary = 2 - distMod + modifierTotal
    pub fn minimum_roll_pass_bb2016(agility: i32, distance: PassingDistance, modifier_total: i32) -> i32 {
        let dist_mod = distance.modifier_2016();
        let ag_capped = agility.min(6);
        let ag_based = 7 - ag_capped - dist_mod + modifier_total;
        let fumble_boundary = 2 - dist_mod + modifier_total;
        ag_based.max(fumble_boundary).max(2)
    }

    /// Minimum roll for a pass in BB2020/BB2025.
    /// Returns None when the player cannot pass (passing_ability <= 0).
    pub fn minimum_roll_pass_bb2020(passing_ability: i32, distance: PassingDistance, modifier_total: i32) -> Option<i32> {
        if passing_ability <= 0 {
            return None;
        }
        Some(2_i32.max(passing_ability + distance.modifier_2020() + modifier_total))
    }

    /// Whether a BB2016 pass roll is a modified fumble.
    /// Fumble when: roll + dist_mod - modifiers <= 1
    pub fn is_modified_fumble_bb2016(roll: i32, distance: PassingDistance, modifier_total: i32) -> bool {
        (roll + distance.modifier_2016() - modifier_total) <= 1
    }
}

impl Default for PassCalc {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimum_roll_pass_bb2016_ag3_short_pass_no_mods() {
        // ag=3, distMod=0 (ShortPass), agBased=7-3-0+0=4, fumbleBoundary=2-0+0=2
        // max(4,2,2) = 4
        assert_eq!(PassCalc::minimum_roll_pass_bb2016(3, PassingDistance::ShortPass, 0), 4);
    }

    #[test]
    fn minimum_roll_pass_bb2016_ag3_long_bomb_no_mods() {
        // distMod=-2, ag_capped=3, agBased=7-3-(-2)+0=6, fumbleBoundary=2-(-2)+0=4
        // max(6,4,2) = 6
        assert_eq!(PassCalc::minimum_roll_pass_bb2016(3, PassingDistance::LongBomb, 0), 6);
    }

    #[test]
    fn minimum_roll_pass_bb2016_floors_at_2() {
        // ag=6 (base=1), QuickPass (distMod=1): agBased=7-6-1+0=0, fumbleBoundary=2-1+0=1
        // max(0,1,2) = 2
        assert_eq!(PassCalc::minimum_roll_pass_bb2016(6, PassingDistance::QuickPass, 0), 2);
    }

    #[test]
    fn minimum_roll_pass_bb2020_none_for_no_passing() {
        assert_eq!(PassCalc::minimum_roll_pass_bb2020(0, PassingDistance::ShortPass, 0), None);
    }

    #[test]
    fn minimum_roll_pass_bb2020_none_for_negative() {
        assert_eq!(PassCalc::minimum_roll_pass_bb2020(-1, PassingDistance::ShortPass, 0), None);
    }

    #[test]
    fn minimum_roll_pass_bb2020_short_pass_pa3() {
        // pa=3, distMod=1 (ShortPass), 3+1+0=4
        assert_eq!(PassCalc::minimum_roll_pass_bb2020(3, PassingDistance::ShortPass, 0), Some(4));
    }

    #[test]
    fn minimum_roll_pass_bb2020_floors_at_2() {
        // pa=2, QuickPass (distMod=0), mod=-5: 2+0-5=-3 → max(2,-3)=2
        assert_eq!(PassCalc::minimum_roll_pass_bb2020(2, PassingDistance::QuickPass, -5), Some(2));
    }

    #[test]
    fn is_modified_fumble_bb2016_roll_1_short_pass() {
        // roll=1, distMod=0, mod=0: 1+0-0=1 ≤ 1 → true
        assert!(PassCalc::is_modified_fumble_bb2016(1, PassingDistance::ShortPass, 0));
    }

    #[test]
    fn is_modified_fumble_bb2016_roll_2_not_fumble() {
        // roll=2, distMod=0, mod=0: 2+0-0=2 > 1 → false
        assert!(!PassCalc::is_modified_fumble_bb2016(2, PassingDistance::ShortPass, 0));
    }

    #[test]
    fn is_modified_fumble_bb2016_long_bomb_roll_2() {
        // LongBomb distMod=-2: roll=2, 2+(-2)-0=0 ≤ 1 → true (fumble!)
        assert!(PassCalc::is_modified_fumble_bb2016(2, PassingDistance::LongBomb, 0));
    }
}
