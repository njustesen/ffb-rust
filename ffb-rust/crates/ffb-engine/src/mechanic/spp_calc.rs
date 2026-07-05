/// 1:1 translation of `com.fumbbl.ffb.server.mechanic.SppCalc`.
///
/// Pure SPP (Star Player Points) award values by edition.
///
/// BB2016:  TD=3, CAS=2, COMP=1, INT=2, DEFL=1, CATCH=1, LANDING=0, MVP=5
/// BB2020:  TD=3, CAS=2, COMP=1, INT=2, DEFL=1, CATCH=1, LANDING=0, MVP=4
///          Teams can earn +1 additional for CAS/COMP/CATCH (league setting)
/// BB2025:  Same as BB2020 but LANDING=1; BrawlinBrutes: TD=2, CAS=3
use ffb_model::enums::Rules;

pub struct SppCalc;

impl SppCalc {
    pub fn touchdown_spp(rules: Rules, is_brawlin_brutes: bool) -> i32 {
        if rules == Rules::Bb2025 && is_brawlin_brutes { return 2; }
        3
    }

    pub fn touchdown_spp_basic(rules: Rules) -> i32 {
        Self::touchdown_spp(rules, false)
    }

    pub fn casualty_spp(rules: Rules, is_brawlin_brutes: bool) -> i32 {
        if rules == Rules::Bb2025 && is_brawlin_brutes { return 3; }
        2
    }

    pub fn casualty_spp_basic(rules: Rules) -> i32 {
        Self::casualty_spp(rules, false)
    }

    pub fn completion_spp() -> i32 { 1 }

    pub fn interception_spp() -> i32 { 2 }

    pub fn deflection_spp() -> i32 { 1 }

    pub fn catch_spp() -> i32 { 1 }

    pub fn landing_spp(rules: Rules) -> i32 {
        if rules == Rules::Bb2025 { 1 } else { 0 }
    }

    pub fn mvp_spp(rules: Rules) -> i32 {
        if rules == Rules::Bb2016 { 5 } else { 4 }
    }

    /// Additional SPP awarded per casualty/completion/catch when the team has the
    /// league "additional SPP" bonus (BB2020/BB2025 only).
    pub fn additional_spp(rules: Rules) -> i32 {
        if rules == Rules::Bb2016 { 0 } else { 1 }
    }

    /// SPP thresholds at which a player advances to the next level.
    /// Applies to BB2016 only (BB2020/BB2025 level by skills gained).
    ///
    /// 6, 16, 31, 51, 76, 176 (inclusive, i.e. ≥ 6 = Experienced)
    pub const LEVEL_THRESHOLDS_BB2016: [i32; 6] = [6, 16, 31, 51, 76, 176];

    /// Player level (0=Rookie, 1=Experienced, ..., 6=Legend) from SPP total (BB2016 only).
    pub fn player_level_bb2016(current_spp: i32) -> i32 {
        let mut level = 0;
        for &threshold in &Self::LEVEL_THRESHOLDS_BB2016 {
            if current_spp >= threshold {
                level += 1;
            } else {
                break;
            }
        }
        level
    }

    /// Whether the player just levelled up given old and new SPP totals (BB2016).
    pub fn just_levelled_up_bb2016(old_spp: i32, new_spp: i32) -> bool {
        Self::player_level_bb2016(old_spp) < Self::player_level_bb2016(new_spp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn touchdown_spp_normal_is_3() {
        assert_eq!(SppCalc::touchdown_spp_basic(Rules::Bb2025), 3);
        assert_eq!(SppCalc::touchdown_spp_basic(Rules::Bb2020), 3);
        assert_eq!(SppCalc::touchdown_spp_basic(Rules::Bb2016), 3);
    }

    #[test]
    fn touchdown_spp_brawlin_brutes_bb2025_is_2() {
        assert_eq!(SppCalc::touchdown_spp(Rules::Bb2025, true), 2);
    }

    #[test]
    fn touchdown_spp_brawlin_brutes_non_bb2025_is_3() {
        assert_eq!(SppCalc::touchdown_spp(Rules::Bb2020, true), 3);
        assert_eq!(SppCalc::touchdown_spp(Rules::Bb2016, true), 3);
    }

    #[test]
    fn casualty_spp_normal_is_2() {
        assert_eq!(SppCalc::casualty_spp_basic(Rules::Bb2025), 2);
        assert_eq!(SppCalc::casualty_spp_basic(Rules::Bb2016), 2);
    }

    #[test]
    fn casualty_spp_brawlin_brutes_bb2025_is_3() {
        assert_eq!(SppCalc::casualty_spp(Rules::Bb2025, true), 3);
    }

    #[test]
    fn fixed_spp_values_all_editions() {
        assert_eq!(SppCalc::completion_spp(), 1);
        assert_eq!(SppCalc::interception_spp(), 2);
        assert_eq!(SppCalc::deflection_spp(), 1);
        assert_eq!(SppCalc::catch_spp(), 1);
    }

    #[test]
    fn landing_spp_bb2025_is_1() {
        assert_eq!(SppCalc::landing_spp(Rules::Bb2025), 1);
    }

    #[test]
    fn landing_spp_bb2016_bb2020_is_0() {
        assert_eq!(SppCalc::landing_spp(Rules::Bb2016), 0);
        assert_eq!(SppCalc::landing_spp(Rules::Bb2020), 0);
    }

    #[test]
    fn mvp_spp_bb2016_is_5() {
        assert_eq!(SppCalc::mvp_spp(Rules::Bb2016), 5);
    }

    #[test]
    fn mvp_spp_bb2020_bb2025_is_4() {
        assert_eq!(SppCalc::mvp_spp(Rules::Bb2020), 4);
        assert_eq!(SppCalc::mvp_spp(Rules::Bb2025), 4);
    }

    #[test]
    fn additional_spp_bb2016_is_0() {
        assert_eq!(SppCalc::additional_spp(Rules::Bb2016), 0);
    }

    #[test]
    fn additional_spp_bb2020_bb2025_is_1() {
        assert_eq!(SppCalc::additional_spp(Rules::Bb2020), 1);
        assert_eq!(SppCalc::additional_spp(Rules::Bb2025), 1);
    }

    #[test]
    fn player_level_bb2016_rookie_at_zero() {
        assert_eq!(SppCalc::player_level_bb2016(0), 0);
        assert_eq!(SppCalc::player_level_bb2016(5), 0);
    }

    #[test]
    fn player_level_bb2016_experienced_at_6() {
        assert_eq!(SppCalc::player_level_bb2016(6), 1);
        assert_eq!(SppCalc::player_level_bb2016(15), 1);
    }

    #[test]
    fn player_level_bb2016_legend_at_176() {
        assert_eq!(SppCalc::player_level_bb2016(176), 6);
        assert_eq!(SppCalc::player_level_bb2016(9999), 6);
    }

    #[test]
    fn player_level_bb2016_all_thresholds() {
        // Level 2 at 16, level 3 at 31, level 4 at 51, level 5 at 76
        assert_eq!(SppCalc::player_level_bb2016(16), 2);
        assert_eq!(SppCalc::player_level_bb2016(31), 3);
        assert_eq!(SppCalc::player_level_bb2016(51), 4);
        assert_eq!(SppCalc::player_level_bb2016(76), 5);
    }

    #[test]
    fn just_levelled_up_bb2016_detects_transition() {
        assert!(SppCalc::just_levelled_up_bb2016(5, 6));
        assert!(!SppCalc::just_levelled_up_bb2016(6, 10));
        assert!(SppCalc::just_levelled_up_bb2016(15, 16));
    }
}
