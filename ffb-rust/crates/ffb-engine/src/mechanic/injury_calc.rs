/// Pure injury-roll interpretation extracted from edition-specific RollMechanic classes.
///
/// All editions share the same base table; Stunty and Thick Skull interactions differ
/// between BB2016 and BB2020/BB2025.
///
/// 1:1 translation of `com.fumbbl.ffb.server.mechanic.InjuryCalc`.
pub struct InjuryCalc;

// PlayerState constants matching Java's PlayerState.STUNNED / KNOCKED_OUT / BADLY_HURT
pub const PLAYER_STATE_STUNNED: i32 = 3;
pub const PLAYER_STATE_KNOCKED_OUT: i32 = 4;
pub const PLAYER_STATE_BADLY_HURT: i32 = 7;

impl InjuryCalc {
    /// Interprets an injury roll total for BB2016 rules.
    ///
    /// - 8 + Thick Skull → Stunned
    /// - 7 + Stunty → KO
    /// - 9 + Stunty → Badly Hurt
    /// - 10+ → casualty (returns `None`)
    /// - 8–9 → KO
    /// - 2–7 → Stunned
    pub fn interpret_injury_total_bb2016(total: i32, is_stunty: bool, has_thick_skull: bool) -> Option<i32> {
        if total == 8 && has_thick_skull { return Some(PLAYER_STATE_STUNNED); }
        if total == 7 && is_stunty { return Some(PLAYER_STATE_KNOCKED_OUT); }
        if total == 9 && is_stunty { return Some(PLAYER_STATE_BADLY_HURT); }
        if total > 9 { return None; }
        if total > 7 { return Some(PLAYER_STATE_KNOCKED_OUT); }
        Some(PLAYER_STATE_STUNNED)
    }

    /// Interprets an injury roll total for BB2020/BB2025 rules.
    ///
    /// - 7 + Stunty + Thick Skull → Stunned
    /// - 7 + Stunty → KO
    /// - 8 + Thick Skull (non-Stunty) → Stunned
    /// - 9 + Stunty → Badly Hurt
    /// - 10+ → casualty (returns `None`)
    /// - 8–9 → KO
    /// - 2–7 → Stunned
    pub fn interpret_injury_total_bb2020(total: i32, is_stunty: bool, has_thick_skull: bool) -> Option<i32> {
        if total == 7 && is_stunty {
            return Some(if has_thick_skull { PLAYER_STATE_STUNNED } else { PLAYER_STATE_KNOCKED_OUT });
        }
        if total == 8 && has_thick_skull && !is_stunty { return Some(PLAYER_STATE_STUNNED); }
        if total == 9 && is_stunty { return Some(PLAYER_STATE_BADLY_HURT); }
        if total > 9 { return None; }
        if total > 7 { return Some(PLAYER_STATE_KNOCKED_OUT); }
        Some(PLAYER_STATE_STUNNED)
    }
}

#[cfg(test)]
mod tests {
    // 1:1 mirror of com.fumbbl.ffb.server.mechanic.InjuryCalcTest
    // (Java @ParameterizedTest value sources become in-test loops).
    use super::*;

    // ══════════════════════════════════════════════════════════════════════
    // BB2016
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn bb2016_low_total_is_stunned() {
        for total in [2, 3, 4, 5, 6, 7] {
            assert_eq!(
                InjuryCalc::interpret_injury_total_bb2016(total, false, false),
                Some(PLAYER_STATE_STUNNED),
                "total={total}"
            );
        }
    }

    #[test]
    fn bb2016_mid_total_is_ko() {
        for total in [8, 9] {
            assert_eq!(
                InjuryCalc::interpret_injury_total_bb2016(total, false, false),
                Some(PLAYER_STATE_KNOCKED_OUT),
                "total={total}"
            );
        }
    }

    #[test]
    fn bb2016_high_total_is_casualty() {
        for total in [10, 11, 12] {
            assert_eq!(InjuryCalc::interpret_injury_total_bb2016(total, false, false), None, "total={total}");
        }
    }

    #[test]
    fn bb2016_thick_skull_at8_becomes_stunned() {
        assert_eq!(InjuryCalc::interpret_injury_total_bb2016(8, false, true), Some(PLAYER_STATE_STUNNED));
    }

    #[test]
    fn bb2016_thick_skull_at9_still_ko() {
        assert_eq!(InjuryCalc::interpret_injury_total_bb2016(9, false, true), Some(PLAYER_STATE_KNOCKED_OUT));
    }

    #[test]
    fn bb2016_stunty_at7_becomes_ko() {
        assert_eq!(InjuryCalc::interpret_injury_total_bb2016(7, true, false), Some(PLAYER_STATE_KNOCKED_OUT));
    }

    #[test]
    fn bb2016_stunty_at9_becomes_badly_hurt() {
        assert_eq!(InjuryCalc::interpret_injury_total_bb2016(9, true, false), Some(PLAYER_STATE_BADLY_HURT));
    }

    #[test]
    fn bb2016_thick_skull_at8_overrides_stunty_because_bb2016_checks_thick_skull_first() {
        // In BB2016 Thick Skull at 8 is checked BEFORE Stunty — so even a Stunty player with
        // ThickSkull is Stunned at total 8, not KO.
        assert_eq!(InjuryCalc::interpret_injury_total_bb2016(8, true, true), Some(PLAYER_STATE_STUNNED));
    }

    #[test]
    fn bb2016_stunty_thick_skull_at7_is_ko_because_thick_skull_only_activates_at8() {
        // ThickSkull only saves at 8 in BB2016; at 7 with Stunty it's still KO
        assert_eq!(InjuryCalc::interpret_injury_total_bb2016(7, true, true), Some(PLAYER_STATE_KNOCKED_OUT));
    }

    // ══════════════════════════════════════════════════════════════════════
    // BB2020 / BB2025 (share same logic)
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn bb2020_low_total_is_stunned() {
        for total in [2, 3, 4, 5, 6, 7] {
            assert_eq!(
                InjuryCalc::interpret_injury_total_bb2020(total, false, false),
                Some(PLAYER_STATE_STUNNED),
                "total={total}"
            );
        }
    }

    #[test]
    fn bb2020_mid_total_is_ko() {
        for total in [8, 9] {
            assert_eq!(
                InjuryCalc::interpret_injury_total_bb2020(total, false, false),
                Some(PLAYER_STATE_KNOCKED_OUT),
                "total={total}"
            );
        }
    }

    #[test]
    fn bb2020_high_total_is_casualty() {
        for total in [10, 11, 12] {
            assert_eq!(InjuryCalc::interpret_injury_total_bb2020(total, false, false), None, "total={total}");
        }
    }

    #[test]
    fn bb2020_thick_skull_at8_non_stunty_becomes_stunned() {
        assert_eq!(InjuryCalc::interpret_injury_total_bb2020(8, false, true), Some(PLAYER_STATE_STUNNED));
    }

    #[test]
    fn bb2020_stunty_at7_becomes_ko() {
        assert_eq!(InjuryCalc::interpret_injury_total_bb2020(7, true, false), Some(PLAYER_STATE_KNOCKED_OUT));
    }

    #[test]
    fn bb2020_stunty_at9_becomes_badly_hurt() {
        assert_eq!(InjuryCalc::interpret_injury_total_bb2020(9, true, false), Some(PLAYER_STATE_BADLY_HURT));
    }

    #[test]
    fn bb2020_stunty_thick_skull_at7_thick_skull_saves() {
        // BB2020: Thick Skull overrides Stunty at 7 — Stunned instead of KO
        assert_eq!(InjuryCalc::interpret_injury_total_bb2020(7, true, true), Some(PLAYER_STATE_STUNNED));
    }

    #[test]
    fn bb2020_stunty_thick_skull_at8_thick_skull_does_not_save() {
        // BB2020: Thick Skull only saves non-Stunty at 8; Stunty+ThickSkull at 8 → KO
        assert_eq!(InjuryCalc::interpret_injury_total_bb2020(8, true, true), Some(PLAYER_STATE_KNOCKED_OUT));
    }

    #[test]
    fn bb2020_stunty_at10_is_casualty() {
        assert_eq!(InjuryCalc::interpret_injury_total_bb2020(10, true, false), None);
    }
}
