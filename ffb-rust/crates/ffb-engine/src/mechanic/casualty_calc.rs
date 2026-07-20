/// Pure casualty-roll interpretation extracted from edition-specific RollMechanic classes.
///
/// BB2016: rolls a 2d6 casualty die; only the first die determines the outcome tier.
/// BB2020/BB2025: rolls a d16; the full value selects the tier.
/// When the tier is Serious Injury, a separate d6 SI roll further determines the specific injury.
///
/// 1:1 translation of `com.fumbbl.ffb.server.mechanic.CasualtyCalc`.
pub struct CasualtyCalc;

impl CasualtyCalc {
    // ── Tier from casualty die ──────────────────────────────────────────────

    /// BB2016: interprets the first die of the 2d6 casualty roll.
    /// - 6 → RIP
    /// - 4–5 → Serious Injury (requires SI table roll)
    /// - 1–3 → Badly Hurt
    pub fn casualty_tier_bb2016(first_die: i32) -> i32 {
        if first_die == 6 { return PLAYER_STATE_RIP; }
        if first_die >= 4 { return PLAYER_STATE_SERIOUS_INJURY; }
        PLAYER_STATE_BADLY_HURT
    }

    /// BB2020: interprets a d16 casualty roll (modifiers already applied).
    /// - 15+ → RIP
    /// - 7–14 → Serious Injury
    /// - 1–6 → Badly Hurt
    pub fn casualty_tier_bb2020(roll: i32) -> i32 {
        if roll >= 15 { return PLAYER_STATE_RIP; }
        if roll >= 7 { return PLAYER_STATE_SERIOUS_INJURY; }
        PLAYER_STATE_BADLY_HURT
    }

    /// BB2025: interprets a d16 casualty roll (modifiers already applied).
    /// - 15+ → RIP
    /// - 9–14 → Serious Injury
    /// - 1–8 → Badly Hurt
    pub fn casualty_tier_bb2025(roll: i32) -> i32 {
        if roll >= 15 { return PLAYER_STATE_RIP; }
        if roll >= 9 { return PLAYER_STATE_SERIOUS_INJURY; }
        PLAYER_STATE_BADLY_HURT
    }

    // ── SI sub-table ────────────────────────────────────────────────────────

    /// BB2016: returns whether a casualty die first-value triggers the SI detail table.
    /// SI detail table is used when first die is 4 or 5.
    pub fn requires_si_roll_bb2016(first_die: i32) -> bool {
        first_die == 4 || first_die == 5
    }

    /// BB2020/BB2025: returns whether a d16 casualty roll value triggers the SI detail table.
    /// SI detail table is used when the roll is 13 or 14.
    pub fn requires_si_roll_bb2020(roll: i32) -> bool {
        roll == 13 || roll == 14
    }

    /// BB2020/BB2025: sub-type for serious injury when not on the SI detail table.
    pub fn serious_injury_sub_type_bb2020(roll: i32) -> Option<&'static str> {
        if roll >= 10 && roll <= 12 { return Some("SERIOUS_INJURY"); }
        if roll >= 7 && roll <= 9 { return Some("SERIOUSLY_HURT"); }
        None
    }

    /// BB2025: sub-type for serious injury when not on the SI detail table.
    pub fn serious_injury_sub_type_bb2025(roll: i32) -> Option<&'static str> {
        if roll >= 11 && roll <= 12 { return Some("SERIOUS_INJURY"); }
        if roll >= 9 && roll <= 10 { return Some("SERIOUSLY_HURT"); }
        None
    }
}

// PlayerState constants matching Java's PlayerState.RIP / SERIOUS_INJURY / BADLY_HURT
pub const PLAYER_STATE_RIP: i32 = 9;
pub const PLAYER_STATE_SERIOUS_INJURY: i32 = 8;
pub const PLAYER_STATE_BADLY_HURT: i32 = 7;

#[cfg(test)]
mod tests {
    // 1:1 mirror of com.fumbbl.ffb.server.mechanic.CasualtyCalcTest
    // (Java @ParameterizedTest value sources become in-test loops).
    use super::*;

    // ══════════════════════════════════════════════════════════════════════
    // BB2016 — 2d6 casualty die (only first die matters for tier)
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn bb2016_first_die1to3_is_badly_hurt() {
        for die in [1, 2, 3] {
            assert_eq!(CasualtyCalc::casualty_tier_bb2016(die), PLAYER_STATE_BADLY_HURT, "die={die}");
        }
    }

    #[test]
    fn bb2016_first_die4or5_is_serious_injury() {
        for die in [4, 5] {
            assert_eq!(CasualtyCalc::casualty_tier_bb2016(die), PLAYER_STATE_SERIOUS_INJURY, "die={die}");
        }
    }

    #[test]
    fn bb2016_first_die6_is_rip() {
        assert_eq!(CasualtyCalc::casualty_tier_bb2016(6), PLAYER_STATE_RIP);
    }

    #[test]
    fn bb2016_requires_si_roll_only_for4and5() {
        for die in [4, 5] {
            assert!(CasualtyCalc::requires_si_roll_bb2016(die), "die={die}");
        }
    }

    #[test]
    fn bb2016_no_si_roll_for1to3and6() {
        for die in [1, 2, 3, 6] {
            assert!(!CasualtyCalc::requires_si_roll_bb2016(die), "die={die}");
        }
    }

    // ══════════════════════════════════════════════════════════════════════
    // BB2020 — d16 casualty roll
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn bb2020_roll1to6_is_badly_hurt() {
        for roll in [1, 2, 3, 4, 5, 6] {
            assert_eq!(CasualtyCalc::casualty_tier_bb2020(roll), PLAYER_STATE_BADLY_HURT, "roll={roll}");
        }
    }

    #[test]
    fn bb2020_roll7to14_is_serious_injury() {
        for roll in [7, 8, 9, 10, 11, 12, 13, 14] {
            assert_eq!(CasualtyCalc::casualty_tier_bb2020(roll), PLAYER_STATE_SERIOUS_INJURY, "roll={roll}");
        }
    }

    #[test]
    fn bb2020_roll15plus_is_rip() {
        for roll in [15, 16, 17] {
            assert_eq!(CasualtyCalc::casualty_tier_bb2020(roll), PLAYER_STATE_RIP, "roll={roll}");
        }
    }

    #[test]
    fn bb2020_requires_si_roll_only_for13and14() {
        assert!(!CasualtyCalc::requires_si_roll_bb2020(12));
        assert!(CasualtyCalc::requires_si_roll_bb2020(13));
        assert!(CasualtyCalc::requires_si_roll_bb2020(14));
        assert!(!CasualtyCalc::requires_si_roll_bb2020(15));
    }

    #[test]
    fn bb2020_serious_injury_sub_type_seriously_hurt() {
        assert_eq!(CasualtyCalc::serious_injury_sub_type_bb2020(7), Some("SERIOUSLY_HURT"));
        assert_eq!(CasualtyCalc::serious_injury_sub_type_bb2020(9), Some("SERIOUSLY_HURT"));
    }

    #[test]
    fn bb2020_serious_injury_sub_type_serious_injury() {
        assert_eq!(CasualtyCalc::serious_injury_sub_type_bb2020(10), Some("SERIOUS_INJURY"));
        assert_eq!(CasualtyCalc::serious_injury_sub_type_bb2020(12), Some("SERIOUS_INJURY"));
    }

    #[test]
    fn bb2020_serious_injury_sub_type_null_for_si_table_rolls() {
        assert_eq!(CasualtyCalc::serious_injury_sub_type_bb2020(13), None);
        assert_eq!(CasualtyCalc::serious_injury_sub_type_bb2020(14), None);
    }

    #[test]
    fn bb2020_serious_injury_sub_type_null_below_si_range() {
        assert_eq!(CasualtyCalc::serious_injury_sub_type_bb2020(6), None);
    }

    // ══════════════════════════════════════════════════════════════════════
    // BB2025 — d16 casualty roll (same structure, different thresholds)
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn bb2025_roll1to8_is_badly_hurt() {
        for roll in [1, 2, 3, 4, 5, 6, 7, 8] {
            assert_eq!(CasualtyCalc::casualty_tier_bb2025(roll), PLAYER_STATE_BADLY_HURT, "roll={roll}");
        }
    }

    #[test]
    fn bb2025_roll9to14_is_serious_injury() {
        for roll in [9, 10, 11, 12, 13, 14] {
            assert_eq!(CasualtyCalc::casualty_tier_bb2025(roll), PLAYER_STATE_SERIOUS_INJURY, "roll={roll}");
        }
    }

    #[test]
    fn bb2025_roll15plus_is_rip() {
        for roll in [15, 16] {
            assert_eq!(CasualtyCalc::casualty_tier_bb2025(roll), PLAYER_STATE_RIP, "roll={roll}");
        }
    }

    #[test]
    fn bb2025_serious_injury_sub_type_seriously_hurt() {
        assert_eq!(CasualtyCalc::serious_injury_sub_type_bb2025(9), Some("SERIOUSLY_HURT"));
        assert_eq!(CasualtyCalc::serious_injury_sub_type_bb2025(10), Some("SERIOUSLY_HURT"));
    }

    #[test]
    fn bb2025_serious_injury_sub_type_serious_injury() {
        assert_eq!(CasualtyCalc::serious_injury_sub_type_bb2025(11), Some("SERIOUS_INJURY"));
        assert_eq!(CasualtyCalc::serious_injury_sub_type_bb2025(12), Some("SERIOUS_INJURY"));
    }

    #[test]
    fn bb2025_serious_injury_sub_type_null_for_si_table_rolls() {
        assert_eq!(CasualtyCalc::serious_injury_sub_type_bb2025(13), None);
        assert_eq!(CasualtyCalc::serious_injury_sub_type_bb2025(14), None);
    }

    #[test]
    fn bb2025_serious_injury_sub_type_null_below_si_range() {
        assert_eq!(CasualtyCalc::serious_injury_sub_type_bb2025(8), None);
    }

    // ── Cross-edition comparison ──────────────────────────────────────────

    #[test]
    fn bb2025_has_higher_badly_hurt_threshold_than_bb2020() {
        // BB2020: roll 7 → SI; BB2025: roll 7 → Badly Hurt
        assert_eq!(CasualtyCalc::casualty_tier_bb2020(7), PLAYER_STATE_SERIOUS_INJURY);
        assert_eq!(CasualtyCalc::casualty_tier_bb2025(7), PLAYER_STATE_BADLY_HURT);
    }

    #[test]
    fn bb2025_roll8_is_badly_hurt_unlike_bb2020() {
        assert_eq!(CasualtyCalc::casualty_tier_bb2020(8), PLAYER_STATE_SERIOUS_INJURY);
        assert_eq!(CasualtyCalc::casualty_tier_bb2025(8), PLAYER_STATE_BADLY_HURT);
    }
}
