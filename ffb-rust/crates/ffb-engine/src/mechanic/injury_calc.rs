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
    use super::*;

    // ── BB2016 tests ──────────────────────────────────────────────────────

    #[test]
    fn bb2016_total_2_is_stunned() {
        assert_eq!(InjuryCalc::interpret_injury_total_bb2016(2, false, false), Some(PLAYER_STATE_STUNNED));
    }

    #[test]
    fn bb2016_total_7_is_stunned() {
        assert_eq!(InjuryCalc::interpret_injury_total_bb2016(7, false, false), Some(PLAYER_STATE_STUNNED));
    }

    #[test]
    fn bb2016_total_8_is_ko() {
        assert_eq!(InjuryCalc::interpret_injury_total_bb2016(8, false, false), Some(PLAYER_STATE_KNOCKED_OUT));
    }

    #[test]
    fn bb2016_total_9_is_ko() {
        assert_eq!(InjuryCalc::interpret_injury_total_bb2016(9, false, false), Some(PLAYER_STATE_KNOCKED_OUT));
    }

    #[test]
    fn bb2016_total_10_plus_is_casualty() {
        assert_eq!(InjuryCalc::interpret_injury_total_bb2016(10, false, false), None);
        assert_eq!(InjuryCalc::interpret_injury_total_bb2016(12, false, false), None);
    }

    #[test]
    fn bb2016_total_8_thick_skull_is_stunned() {
        assert_eq!(InjuryCalc::interpret_injury_total_bb2016(8, false, true), Some(PLAYER_STATE_STUNNED));
    }

    #[test]
    fn bb2016_total_7_stunty_is_ko() {
        assert_eq!(InjuryCalc::interpret_injury_total_bb2016(7, true, false), Some(PLAYER_STATE_KNOCKED_OUT));
    }

    #[test]
    fn bb2016_total_9_stunty_is_badly_hurt() {
        assert_eq!(InjuryCalc::interpret_injury_total_bb2016(9, true, false), Some(PLAYER_STATE_BADLY_HURT));
    }

    // ── BB2020 tests ──────────────────────────────────────────────────────

    #[test]
    fn bb2020_total_2_is_stunned() {
        assert_eq!(InjuryCalc::interpret_injury_total_bb2020(2, false, false), Some(PLAYER_STATE_STUNNED));
    }

    #[test]
    fn bb2020_total_7_is_stunned() {
        assert_eq!(InjuryCalc::interpret_injury_total_bb2020(7, false, false), Some(PLAYER_STATE_STUNNED));
    }

    #[test]
    fn bb2020_total_8_is_ko() {
        assert_eq!(InjuryCalc::interpret_injury_total_bb2020(8, false, false), Some(PLAYER_STATE_KNOCKED_OUT));
    }

    #[test]
    fn bb2020_total_9_is_ko() {
        assert_eq!(InjuryCalc::interpret_injury_total_bb2020(9, false, false), Some(PLAYER_STATE_KNOCKED_OUT));
    }

    #[test]
    fn bb2020_total_10_plus_is_casualty() {
        assert_eq!(InjuryCalc::interpret_injury_total_bb2020(10, false, false), None);
    }

    #[test]
    fn bb2020_total_7_stunty_thick_skull_is_stunned() {
        assert_eq!(InjuryCalc::interpret_injury_total_bb2020(7, true, true), Some(PLAYER_STATE_STUNNED));
    }

    #[test]
    fn bb2020_total_7_stunty_no_thick_skull_is_ko() {
        assert_eq!(InjuryCalc::interpret_injury_total_bb2020(7, true, false), Some(PLAYER_STATE_KNOCKED_OUT));
    }

    #[test]
    fn bb2020_total_8_thick_skull_non_stunty_is_stunned() {
        assert_eq!(InjuryCalc::interpret_injury_total_bb2020(8, false, true), Some(PLAYER_STATE_STUNNED));
    }

    #[test]
    fn bb2020_total_8_thick_skull_stunty_is_ko() {
        // Thick Skull only saves non-Stunty players at 8
        assert_eq!(InjuryCalc::interpret_injury_total_bb2020(8, true, true), Some(PLAYER_STATE_KNOCKED_OUT));
    }

    #[test]
    fn bb2020_total_9_stunty_is_badly_hurt() {
        assert_eq!(InjuryCalc::interpret_injury_total_bb2020(9, true, false), Some(PLAYER_STATE_BADLY_HURT));
    }
}
