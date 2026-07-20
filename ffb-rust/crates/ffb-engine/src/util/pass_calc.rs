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

// Test mirror of com.fumbbl.ffb.server.util.PassCalcTest (1:1).
#[cfg(test)]
mod tests {
    use super::*;

    // ── minimum_roll_pass_bb2016 ─────────────────────────────────────────────

    #[test]
    fn bb2016_short_pass_ag3() {
        // AG3, short pass (dist_mod=0): 7 - 3 - 0 = 4
        assert_eq!(PassCalc::minimum_roll_pass_bb2016(3, PassingDistance::ShortPass, 0), 4);
    }

    #[test]
    fn bb2016_long_pass_ag4() {
        // AG4, long pass (dist_mod=-1): 7 - 4 + 1 = 4
        assert_eq!(PassCalc::minimum_roll_pass_bb2016(4, PassingDistance::LongPass, 0), 4);
    }

    #[test]
    fn bb2016_quick_pass_ag6_floors_at_2() {
        // AG6, quick pass (dist_mod=+1): 7 - 6 - 1 = 0 → 2; fumble_boundary = 2 - 1 = 1 → max = 2
        assert_eq!(PassCalc::minimum_roll_pass_bb2016(6, PassingDistance::QuickPass, 0), 2);
    }

    #[test]
    fn bb2016_long_bomb_ag6_fumbleboundary_dominates() {
        // AG6, long bomb (dist_mod=-2): ag_based = 7-6+2 = 3; fumble_boundary = 2+2 = 4
        // max(3, 4, 2) = 4 — fumble boundary dominates, not the agility formula
        assert_eq!(PassCalc::minimum_roll_pass_bb2016(6, PassingDistance::LongBomb, 0), 4);
    }

    #[test]
    fn bb2016_long_bomb_ag4() {
        // AG4, long bomb: ag_based = 7-4+2 = 5; fumble_boundary = 4; max = 5
        assert_eq!(PassCalc::minimum_roll_pass_bb2016(4, PassingDistance::LongBomb, 0), 5);
    }

    #[test]
    fn bb2016_long_bomb_ag3() {
        // AG3, long bomb: ag_based = 7-3+2 = 6; fumble_boundary = 4; max = 6
        assert_eq!(PassCalc::minimum_roll_pass_bb2016(3, PassingDistance::LongBomb, 0), 6);
    }

    #[test]
    fn bb2016_with_positive_modifier_harder() {
        // AG4, short pass, +1 modifier (e.g. rain): 7-4-0+1 = 4; fumble_boundary=2+1=3; max=4
        assert_eq!(PassCalc::minimum_roll_pass_bb2016(4, PassingDistance::ShortPass, 1), 4);
    }

    #[test]
    fn bb2016_with_negative_modifier_easier_floor_at_2() {
        // AG4, short pass, -2 (e.g. strong arm): 7-4-0-2 = 1 → 2; fumble=2-0-2=0 → max = 2
        assert_eq!(PassCalc::minimum_roll_pass_bb2016(4, PassingDistance::ShortPass, -2), 2);
    }

    #[test]
    fn bb2016_ag4_all_distances() {
        // Mirrors the Java @ParameterizedTest CsvSource rows
        let rows = [
            (PassingDistance::QuickPass, 2), // 7-4-1=2, fumble=2-1=1; max=2
            (PassingDistance::ShortPass, 3), // 7-4-0=3, fumble=2; max=3
            (PassingDistance::LongPass, 4),  // 7-4+1=4, fumble=3; max=4
            (PassingDistance::LongBomb, 5),  // 7-4+2=5, fumble=4; max=5
        ];
        for (distance, expected) in rows {
            assert_eq!(
                PassCalc::minimum_roll_pass_bb2016(4, distance, 0),
                expected,
                "distance={distance:?}"
            );
        }
    }

    // ── minimum_roll_pass_bb2020 ─────────────────────────────────────────────

    #[test]
    fn bb2020_no_pa_returns_null() {
        assert_eq!(PassCalc::minimum_roll_pass_bb2020(0, PassingDistance::ShortPass, 0), None);
    }

    #[test]
    fn bb2020_negative_pa_returns_null() {
        assert_eq!(PassCalc::minimum_roll_pass_bb2020(-1, PassingDistance::ShortPass, 0), None);
    }

    #[test]
    fn bb2020_pa2_short_pass() {
        // PA2 + dist_mod 1 = 3
        assert_eq!(PassCalc::minimum_roll_pass_bb2020(2, PassingDistance::ShortPass, 0), Some(3));
    }

    #[test]
    fn bb2020_pa3_short_pass() {
        // PA3 + dist_mod 1 = 4
        assert_eq!(PassCalc::minimum_roll_pass_bb2020(3, PassingDistance::ShortPass, 0), Some(4));
    }

    #[test]
    fn bb2020_pa3_quick_pass() {
        // PA3 + dist_mod 0 = 3
        assert_eq!(PassCalc::minimum_roll_pass_bb2020(3, PassingDistance::QuickPass, 0), Some(3));
    }

    #[test]
    fn bb2020_pa3_long_bomb() {
        // PA3 + dist_mod 3 = 6
        assert_eq!(PassCalc::minimum_roll_pass_bb2020(3, PassingDistance::LongBomb, 0), Some(6));
    }

    #[test]
    fn bb2020_with_modifier() {
        // PA3, short pass (1), +1 rain = 5
        assert_eq!(PassCalc::minimum_roll_pass_bb2020(3, PassingDistance::ShortPass, 1), Some(5));
    }

    #[test]
    fn bb2020_floor_at_2() {
        // PA2 + dist_mod 0 - 5 modifiers = -3 → 2
        assert_eq!(PassCalc::minimum_roll_pass_bb2020(2, PassingDistance::QuickPass, -5), Some(2));
    }

    // ── is_modified_fumble_bb2016 ────────────────────────────────────────────

    #[test]
    fn bb2016_natural_one_quick_pass_not_modified_fumble_handle_separately() {
        // Natural 1 fumble is handled separately as a direct rule, not via is_modified_fumble.
        // roll=1, quick pass dist_mod=+1: 1+1-0=2 > 1 → NOT a modified fumble
        // (caller must check roll==1 as a direct fumble first)
        assert!(!PassCalc::is_modified_fumble_bb2016(1, PassingDistance::QuickPass, 0));
    }

    #[test]
    fn bb2016_long_bomb_roll2_is_fumble() {
        // roll 2 + dist_mod(-2) - 0 = 0 <= 1 → fumble
        assert!(PassCalc::is_modified_fumble_bb2016(2, PassingDistance::LongBomb, 0));
    }

    #[test]
    fn bb2016_short_pass_roll2_not_fumble() {
        // roll 2 + dist_mod(0) - 0 = 2 > 1 → not fumble
        assert!(!PassCalc::is_modified_fumble_bb2016(2, PassingDistance::ShortPass, 0));
    }

    #[test]
    fn bb2016_quick_pass_roll1_not_modified_fumble() {
        // roll 1 + dist_mod(+1) - 0 = 2 > 1 → not a modified fumble
        // (natural 1 is handled separately as a direct fumble regardless)
        assert!(!PassCalc::is_modified_fumble_bb2016(1, PassingDistance::QuickPass, 0));
    }

    #[test]
    fn bb2016_short_pass_roll1_is_fumble() {
        // roll 1 + dist_mod(0) - 0 = 1 <= 1 → fumble
        assert!(PassCalc::is_modified_fumble_bb2016(1, PassingDistance::ShortPass, 0));
    }
}
