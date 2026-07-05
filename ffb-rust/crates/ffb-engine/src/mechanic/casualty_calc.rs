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
    use super::*;

    // ── casualty_tier_bb2016 ─────────────────────────────────────────────

    #[test]
    fn bb2016_die_6_is_rip() {
        assert_eq!(CasualtyCalc::casualty_tier_bb2016(6), PLAYER_STATE_RIP);
    }

    #[test]
    fn bb2016_die_4_is_serious_injury() {
        assert_eq!(CasualtyCalc::casualty_tier_bb2016(4), PLAYER_STATE_SERIOUS_INJURY);
    }

    #[test]
    fn bb2016_die_5_is_serious_injury() {
        assert_eq!(CasualtyCalc::casualty_tier_bb2016(5), PLAYER_STATE_SERIOUS_INJURY);
    }

    #[test]
    fn bb2016_die_1_is_badly_hurt() {
        assert_eq!(CasualtyCalc::casualty_tier_bb2016(1), PLAYER_STATE_BADLY_HURT);
    }

    #[test]
    fn bb2016_die_3_is_badly_hurt() {
        assert_eq!(CasualtyCalc::casualty_tier_bb2016(3), PLAYER_STATE_BADLY_HURT);
    }

    // ── casualty_tier_bb2020 ─────────────────────────────────────────────

    #[test]
    fn bb2020_roll_15_is_rip() {
        assert_eq!(CasualtyCalc::casualty_tier_bb2020(15), PLAYER_STATE_RIP);
    }

    #[test]
    fn bb2020_roll_16_is_rip() {
        assert_eq!(CasualtyCalc::casualty_tier_bb2020(16), PLAYER_STATE_RIP);
    }

    #[test]
    fn bb2020_roll_7_is_serious_injury() {
        assert_eq!(CasualtyCalc::casualty_tier_bb2020(7), PLAYER_STATE_SERIOUS_INJURY);
    }

    #[test]
    fn bb2020_roll_14_is_serious_injury() {
        assert_eq!(CasualtyCalc::casualty_tier_bb2020(14), PLAYER_STATE_SERIOUS_INJURY);
    }

    #[test]
    fn bb2020_roll_6_is_badly_hurt() {
        assert_eq!(CasualtyCalc::casualty_tier_bb2020(6), PLAYER_STATE_BADLY_HURT);
    }

    #[test]
    fn bb2020_roll_1_is_badly_hurt() {
        assert_eq!(CasualtyCalc::casualty_tier_bb2020(1), PLAYER_STATE_BADLY_HURT);
    }

    // ── casualty_tier_bb2025 ─────────────────────────────────────────────

    #[test]
    fn bb2025_roll_15_is_rip() {
        assert_eq!(CasualtyCalc::casualty_tier_bb2025(15), PLAYER_STATE_RIP);
    }

    #[test]
    fn bb2025_roll_9_is_serious_injury() {
        assert_eq!(CasualtyCalc::casualty_tier_bb2025(9), PLAYER_STATE_SERIOUS_INJURY);
    }

    #[test]
    fn bb2025_roll_14_is_serious_injury() {
        assert_eq!(CasualtyCalc::casualty_tier_bb2025(14), PLAYER_STATE_SERIOUS_INJURY);
    }

    #[test]
    fn bb2025_roll_8_is_badly_hurt() {
        assert_eq!(CasualtyCalc::casualty_tier_bb2025(8), PLAYER_STATE_BADLY_HURT);
    }

    #[test]
    fn bb2025_roll_1_is_badly_hurt() {
        assert_eq!(CasualtyCalc::casualty_tier_bb2025(1), PLAYER_STATE_BADLY_HURT);
    }

    // ── requires_si_roll ─────────────────────────────────────────────────

    #[test]
    fn bb2016_die_4_requires_si_roll() {
        assert!(CasualtyCalc::requires_si_roll_bb2016(4));
    }

    #[test]
    fn bb2016_die_5_requires_si_roll() {
        assert!(CasualtyCalc::requires_si_roll_bb2016(5));
    }

    #[test]
    fn bb2016_die_6_does_not_require_si_roll() {
        assert!(!CasualtyCalc::requires_si_roll_bb2016(6));
    }

    #[test]
    fn bb2020_roll_13_requires_si_roll() {
        assert!(CasualtyCalc::requires_si_roll_bb2020(13));
    }

    #[test]
    fn bb2020_roll_14_requires_si_roll() {
        assert!(CasualtyCalc::requires_si_roll_bb2020(14));
    }

    #[test]
    fn bb2020_roll_12_does_not_require_si_roll() {
        assert!(!CasualtyCalc::requires_si_roll_bb2020(12));
    }

    // ── serious_injury_sub_type ──────────────────────────────────────────

    #[test]
    fn bb2020_roll_10_12_is_serious_injury_subtype() {
        for r in 10..=12 {
            assert_eq!(CasualtyCalc::serious_injury_sub_type_bb2020(r), Some("SERIOUS_INJURY"), "roll={r}");
        }
    }

    #[test]
    fn bb2020_roll_7_9_is_seriously_hurt() {
        for r in 7..=9 {
            assert_eq!(CasualtyCalc::serious_injury_sub_type_bb2020(r), Some("SERIOUSLY_HURT"), "roll={r}");
        }
    }

    #[test]
    fn bb2020_roll_6_has_no_subtype() {
        assert_eq!(CasualtyCalc::serious_injury_sub_type_bb2020(6), None);
    }

    #[test]
    fn bb2025_roll_11_12_is_serious_injury_subtype() {
        for r in 11..=12 {
            assert_eq!(CasualtyCalc::serious_injury_sub_type_bb2025(r), Some("SERIOUS_INJURY"), "roll={r}");
        }
    }

    #[test]
    fn bb2025_roll_9_10_is_seriously_hurt() {
        for r in 9..=10 {
            assert_eq!(CasualtyCalc::serious_injury_sub_type_bb2025(r), Some("SERIOUSLY_HURT"), "roll={r}");
        }
    }

    #[test]
    fn bb2025_roll_8_has_no_subtype() {
        assert_eq!(CasualtyCalc::serious_injury_sub_type_bb2025(8), None);
    }
}
