use ffb_model::enums::{Rules, SeriousInjuryKind};
use crate::modifiers::Modifier;

/// Outcome of an injury roll.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InjuryOutcome {
    Stunned,
    KnockedOut,
    /// Stunty at roll 9: direct Badly Hurt, bypasses casualty roll.
    BadlyHurt,
    /// Casualty roll required to determine player fate.
    Casualty,
}

/// Outcome of an armor roll.
///
/// Returns `true` when the armor is broken (roll > armor_value after modifiers).
/// Formula: 2d6 + sum(modifiers) > armor_value
pub fn armor_broken(armor_value: i32, roll: [i32; 2], modifiers: &[Modifier]) -> bool {
    let modifier_sum: i32 = modifiers.iter().map(|m| m.value).sum();
    roll[0] + roll[1] + modifier_sum >= armor_value
}

/// Evaluate the injury table from a 2d6 roll (all editions share the same table).
///
/// 2–7  → Stunned
/// 8–9  → Knocked Out
/// 10+  → Casualty
pub fn injury_result(roll: [i32; 2], modifiers: &[Modifier]) -> InjuryOutcome {
    let modifier_sum: i32 = modifiers.iter().map(|m| m.value).sum();
    let total = roll[0] + roll[1] + modifier_sum;
    match total {
        i32::MIN..=7 => InjuryOutcome::Stunned,
        8..=9 => InjuryOutcome::KnockedOut,
        _ => InjuryOutcome::Casualty,
    }
}

/// Whether the injury roll total results in at least a Knock Out.
pub fn is_ko_or_worse(roll: [i32; 2], modifiers: &[Modifier]) -> bool {
    !matches!(injury_result(roll, modifiers), InjuryOutcome::Stunned)
}

/// Whether an apothecary can be used (all editions: only on KO or Casualty results).
pub fn apo_usable_for_injury(_rules: Rules, outcome: InjuryOutcome) -> bool {
    matches!(outcome, InjuryOutcome::KnockedOut | InjuryOutcome::Casualty)
}

/// Interpret an injury roll total for **BB2016**.
///
/// Thick Skull is checked at 8 *before* Stunty, so a Stunty+ThickSkull player at 8
/// is still Stunned (BB2016 behaviour differs from BB2020 on this edge case).
///
/// Returns `None` when the total reaches the casualty threshold (10+).
pub fn interpret_injury_total_bb2016(
    total: i32,
    is_stunty: bool,
    has_thick_skull: bool,
) -> Option<InjuryOutcome> {
    if total == 8 && has_thick_skull {
        return Some(InjuryOutcome::Stunned);
    }
    if total == 7 && is_stunty {
        return Some(InjuryOutcome::KnockedOut);
    }
    if total == 9 && is_stunty {
        return Some(InjuryOutcome::BadlyHurt);
    }
    if total > 9 {
        return None;
    }
    if total > 7 {
        return Some(InjuryOutcome::KnockedOut);
    }
    Some(InjuryOutcome::Stunned)
}

/// Interpret an injury roll total for **BB2020 / BB2025**.
///
/// Stunty is checked at 7 first; Thick Skull can override it (unlike BB2016).
/// At 8, Thick Skull only saves non-Stunty players.
///
/// Returns `None` when the total reaches the casualty threshold (10+).
pub fn interpret_injury_total_bb2020(
    total: i32,
    is_stunty: bool,
    has_thick_skull: bool,
) -> Option<InjuryOutcome> {
    if total == 7 && is_stunty {
        return Some(if has_thick_skull {
            InjuryOutcome::Stunned
        } else {
            InjuryOutcome::KnockedOut
        });
    }
    if total == 8 && has_thick_skull && !is_stunty {
        return Some(InjuryOutcome::Stunned);
    }
    if total == 9 && is_stunty {
        return Some(InjuryOutcome::BadlyHurt);
    }
    if total > 9 {
        return None;
    }
    if total > 7 {
        return Some(InjuryOutcome::KnockedOut);
    }
    Some(InjuryOutcome::Stunned)
}

// ─── Casualty roll → player-state tier ───────────────────────────────────────

/// BB2016 casualty: only the **first die** (1d6) determines the tier.
///
/// 6 → Dead/Removed; 4–5 → Serious Injury; 1–3 → Badly Hurt.
pub fn casualty_tier_bb2016(first_die: i32) -> CasualtyTier {
    if first_die == 6 {
        CasualtyTier::Dead
    } else if first_die >= 4 {
        CasualtyTier::SeriousInjury
    } else {
        CasualtyTier::BadlyHurt
    }
}

/// BB2020 casualty: a **d16** roll (modifiers already applied).
///
/// 15+ → Dead; 7–14 → Serious Injury; 1–6 → Badly Hurt.
pub fn casualty_tier_bb2020(roll: i32) -> CasualtyTier {
    if roll >= 15 {
        CasualtyTier::Dead
    } else if roll >= 7 {
        CasualtyTier::SeriousInjury
    } else {
        CasualtyTier::BadlyHurt
    }
}

/// BB2025 casualty: a **d16** roll (modifiers already applied).
///
/// 15+ → Dead; 9–14 → Serious Injury; 1–8 → Badly Hurt.
pub fn casualty_tier_bb2025(roll: i32) -> CasualtyTier {
    if roll >= 15 {
        CasualtyTier::Dead
    } else if roll >= 9 {
        CasualtyTier::SeriousInjury
    } else {
        CasualtyTier::BadlyHurt
    }
}

/// BB2025 d16 casualty roll → specific SeriousInjuryKind (None = Badly Hurt).
pub fn serious_injury_kind_bb2025(roll: i32) -> Option<SeriousInjuryKind> {
    match roll {
        1..=8  => None,
        9      => Some(SeriousInjuryKind::SmashedKneeMa),
        10     => Some(SeriousInjuryKind::HeadInjuryAv),
        11     => Some(SeriousInjuryKind::BrokenArmPa),
        12     => Some(SeriousInjuryKind::NeckInjuryAg),
        13     => Some(SeriousInjuryKind::DislocatedHipAg),
        14     => Some(SeriousInjuryKind::DislocatedShoulderSt),
        _      => Some(SeriousInjuryKind::Dead),
    }
}

/// High-level casualty outcome tier — determines what happens to the player.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CasualtyTier {
    BadlyHurt,
    SeriousInjury,
    Dead,
}

/// Whether a casualty roll requires the SI detail table (stat-reduction lookup).
///
/// BB2020/BB2025: rolls 13 and 14 trigger the d6 SI table.
pub fn requires_si_table_bb2020(roll: i32) -> bool {
    roll == 13 || roll == 14
}

/// BB2020 SI sub-type for rolls that do *not* use the SI detail table.
///
/// Returns `None` for rolls ≤6 (Badly Hurt tier) or 13–14 (SI detail table).
pub fn si_sub_type_bb2020(roll: i32) -> Option<SiSubType> {
    match roll {
        10..=12 => Some(SiSubType::SeriousInjury),
        7..=9 => Some(SiSubType::SeriouslyHurt),
        _ => None,
    }
}

/// BB2025 SI sub-type for rolls that do *not* use the SI detail table.
pub fn si_sub_type_bb2025(roll: i32) -> Option<SiSubType> {
    match roll {
        11..=12 => Some(SiSubType::SeriousInjury),
        9..=10 => Some(SiSubType::SeriouslyHurt),
        _ => None,
    }
}

/// Whether a BB2016 casualty die first-value requires the SI detail table.
pub fn requires_si_table_bb2016(first_die: i32) -> bool {
    first_die == 4 || first_die == 5
}

/// Sub-classification within the SeriousInjury tier (BB2020/BB2025).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SiSubType {
    SeriouslyHurt,
    SeriousInjury,
}

// ─── BB2016 serious injury table (2d6) ────────────────────────────────────────

/// BB2016 SI table lookup by `[die1, die2]`.
///
/// die1=4: all 8 entries are MNG (miss next game, no stat reduction).
/// die1=5: NI or stat-reducing injuries.
/// All other die1 values → `None` (Badly Hurt at die1=1–3; Dead at die1=6).
///
/// Note: BB2016 has unique injury names not all representable in the shared enum.
/// MNG injuries with no stat effect use the closest named MNG variant available.
/// (4,7)=SMASHED_HAND and (4,8)=PINCHED_NERVE share PinchedNerve; (4,5)=FRACTURED_ARM
/// and (4,6)=FRACTURED_LEG use BrokenShoulder/SmashedElbow as stand-ins.
pub fn serious_injury_bb2016(die1: i32, die2: i32) -> Option<SeriousInjuryKind> {
    match (die1, die2) {
        (4, 1) => Some(SeriousInjuryKind::BrokenRibs),       // MNG
        (4, 2) => Some(SeriousInjuryKind::Groin),             // MNG
        (4, 3) => Some(SeriousInjuryKind::BrokenJaw),         // MNG (GOUGED_EYE in Java)
        (4, 4) => Some(SeriousInjuryKind::BrokenJaw),         // MNG
        (4, 5) => Some(SeriousInjuryKind::BrokenShoulder),    // MNG (FRACTURED_ARM)
        (4, 6) => Some(SeriousInjuryKind::SmashedElbow),      // MNG (FRACTURED_LEG)
        (4, 7) => Some(SeriousInjuryKind::ShatteredWrist),    // MNG (SMASHED_HAND)
        (4, 8) => Some(SeriousInjuryKind::PinchedNerve),      // MNG, -AV (PINCHED_NERVE)
        (5, 1) => Some(SeriousInjuryKind::SeriousInjuryNi),   // NI (DAMAGED_BACK)
        (5, 2) => Some(SeriousInjuryKind::SmashedKneeB2016),  // -MA
        (5, 3) => Some(SeriousInjuryKind::SmashedHip),        // -MA (SMASHED_HIP)
        (5, 4) => Some(SeriousInjuryKind::SmashedAnkle),      // -MA (SMASHED_ANKLE)
        (5, 5) => Some(SeriousInjuryKind::HeadInjuryAv),      // -AV (SERIOUS_CONCUSSION)
        (5, 6) => Some(SeriousInjuryKind::HeadInjuryAv),      // -AV (FRACTURED_SKULL)
        (5, 7) => Some(SeriousInjuryKind::NeckInjuryAg),      // -AG (BROKEN_NECK; GougedEye reused for -AG)
        (5, 8) => Some(SeriousInjuryKind::BrokenCollarBone),  // -ST (SMASHED_COLLAR_BONE)
        _ => None,
    }
}

// ─── BB2020/BB2025 serious injury table (d6) ─────────────────────────────────

/// BB2020 SI detail table: maps a d6 roll to a SeriousInjuryKind.
///
/// Used when the d16 casualty roll is 13 or 14.
/// Roll 1–6: HEAD_INJURY, HEAD_INJURY, SMASHED_KNEE, BROKEN_ARM, NECK_INJURY, DISLOCATED_SHOULDER
pub fn serious_injury_bb2020(si_roll: i32) -> Option<SeriousInjuryKind> {
    match si_roll {
        1 | 2 => Some(SeriousInjuryKind::HeadInjuryAv),
        3 => Some(SeriousInjuryKind::SmashedKneeMa),
        4 => Some(SeriousInjuryKind::BrokenArmPa),
        5 => Some(SeriousInjuryKind::NeckInjuryAg),
        6 => Some(SeriousInjuryKind::DislocatedShoulderSt),
        _ => None,
    }
}

/// BB2025 SI detail table: maps a d6 roll to a SeriousInjuryKind.
///
/// Same as BB2020 except roll 5 is DISLOCATED_HIP (-AG) rather than NECK_INJURY (-AG).
/// Roll 1–6: HEAD_INJURY, HEAD_INJURY, SMASHED_KNEE, BROKEN_ARM, DISLOCATED_HIP, DISLOCATED_SHOULDER
pub fn serious_injury_bb2025(si_roll: i32) -> Option<SeriousInjuryKind> {
    match si_roll {
        1 | 2 => Some(SeriousInjuryKind::HeadInjuryAv),
        3 => Some(SeriousInjuryKind::SmashedKneeMa),
        4 => Some(SeriousInjuryKind::BrokenArmPa),
        5 => Some(SeriousInjuryKind::DislocatedHipAg),
        6 => Some(SeriousInjuryKind::DislocatedShoulderSt),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;

    // ── Existing armor / injury tests ─────────────────────────────────────────

    #[test]
    fn armor_broken_exact_threshold() {
        // armor 8, roll 4+5=9 → broken
        assert!(armor_broken(8, [4, 5], &[]));
        // armor 8, roll 4+4=8 → broken (Java: armour <= roll_sum, i.e. 8 <= 8 = true)
        assert!(armor_broken(8, [4, 4], &[]));
        // armor 8, roll 3+4=7 → NOT broken
        assert!(!armor_broken(8, [3, 4], &[]));
    }

    #[test]
    fn armor_broken_with_positive_modifier() {
        // armor 9, roll 4+4=8, +2 mod → 10 > 9 → broken
        let m = Modifier::new("test", 2, Rules::Common);
        assert!(armor_broken(9, [4, 4], &[m]));
    }

    #[test]
    fn armor_not_broken_with_negative_modifier() {
        // armor 8, roll 4+5=9, -2 mod → 7 < 8 → not broken
        let m = Modifier::new("test", -2, Rules::Common);
        assert!(!armor_broken(8, [4, 5], &[m]));
    }

    #[test]
    fn injury_stunned_boundary() {
        assert_eq!(injury_result([1, 1], &[]), InjuryOutcome::Stunned); // 2
        assert_eq!(injury_result([3, 4], &[]), InjuryOutcome::Stunned); // 7
    }

    #[test]
    fn injury_ko_boundary() {
        assert_eq!(injury_result([4, 4], &[]), InjuryOutcome::KnockedOut); // 8
        assert_eq!(injury_result([5, 4], &[]), InjuryOutcome::KnockedOut); // 9
    }

    #[test]
    fn injury_casualty_boundary() {
        assert_eq!(injury_result([5, 5], &[]), InjuryOutcome::Casualty); // 10
        assert_eq!(injury_result([6, 6], &[]), InjuryOutcome::Casualty); // 12
    }

    #[test]
    fn injury_with_modifier_crosses_threshold() {
        // 3+4=7 normally Stunned, +1 → 8 = KO
        let m = Modifier::new("test", 1, Rules::Common);
        assert_eq!(injury_result([3, 4], &[m]), InjuryOutcome::KnockedOut);
    }

    #[test]
    fn apo_usable() {
        assert!(apo_usable_for_injury(Rules::Bb2020, InjuryOutcome::KnockedOut));
        assert!(apo_usable_for_injury(Rules::Bb2020, InjuryOutcome::Casualty));
        assert!(!apo_usable_for_injury(Rules::Bb2020, InjuryOutcome::Stunned));
    }

    // ── interpret_injury_total_bb2016 (names mirror InjuryCalcTest) ───────────

    #[test]
    fn bb2016_low_total_is_stunned() {
        for total in [2, 3, 4, 5, 6, 7] {
            assert_eq!(
                interpret_injury_total_bb2016(total, false, false),
                Some(InjuryOutcome::Stunned),
                "total={total}"
            );
        }
    }

    #[test]
    fn bb2016_mid_total_is_ko() {
        for total in [8, 9] {
            assert_eq!(
                interpret_injury_total_bb2016(total, false, false),
                Some(InjuryOutcome::KnockedOut),
                "total={total}"
            );
        }
    }

    #[test]
    fn bb2016_high_total_is_casualty() {
        for total in [10, 11, 12] {
            assert_eq!(interpret_injury_total_bb2016(total, false, false), None, "total={total}");
        }
    }

    #[test]
    fn bb2016_thick_skull_at8_becomes_stunned() {
        assert_eq!(interpret_injury_total_bb2016(8, false, true), Some(InjuryOutcome::Stunned));
    }

    #[test]
    fn bb2016_thick_skull_at8_overrides_stunty_because_bb2016_checks_thick_skull_first() {
        // BB2016: ThickSkull check comes first, so even Stunty+ThickSkull at 8 → Stunned
        assert_eq!(interpret_injury_total_bb2016(8, true, true), Some(InjuryOutcome::Stunned));
    }

    #[test]
    fn bb2016_stunty_at7_becomes_ko() {
        assert_eq!(interpret_injury_total_bb2016(7, true, false), Some(InjuryOutcome::KnockedOut));
    }

    #[test]
    fn bb2016_stunty_thick_skull_at7_is_ko_because_thick_skull_only_activates_at8() {
        // ThickSkull only triggers at 8 in BB2016, not at 7
        assert_eq!(interpret_injury_total_bb2016(7, true, true), Some(InjuryOutcome::KnockedOut));
    }

    #[test]
    fn bb2016_stunty_at9_becomes_badly_hurt() {
        assert_eq!(interpret_injury_total_bb2016(9, true, false), Some(InjuryOutcome::BadlyHurt));
    }

    // ── interpret_injury_total_bb2020 (names mirror InjuryCalcTest) ───────────

    #[test]
    fn bb2020_low_total_is_stunned() {
        for total in [2, 3, 4, 5, 6, 7] {
            assert_eq!(
                interpret_injury_total_bb2020(total, false, false),
                Some(InjuryOutcome::Stunned),
                "total={total}"
            );
        }
    }

    #[test]
    fn bb2020_mid_total_is_ko() {
        for total in [8, 9] {
            assert_eq!(
                interpret_injury_total_bb2020(total, false, false),
                Some(InjuryOutcome::KnockedOut),
                "total={total}"
            );
        }
    }

    #[test]
    fn bb2020_high_total_is_casualty() {
        for total in [10, 11, 12] {
            assert_eq!(interpret_injury_total_bb2020(total, false, false), None, "total={total}");
        }
    }

    #[test]
    fn bb2020_thick_skull_at8_non_stunty_becomes_stunned() {
        assert_eq!(interpret_injury_total_bb2020(8, false, true), Some(InjuryOutcome::Stunned));
    }

    #[test]
    fn bb2020_stunty_at7_becomes_ko() {
        assert_eq!(interpret_injury_total_bb2020(7, true, false), Some(InjuryOutcome::KnockedOut));
    }

    #[test]
    fn bb2020_stunty_thick_skull_at7_thick_skull_saves() {
        // BB2020: Thick Skull overrides Stunty at 7 — Stunned instead of KO
        assert_eq!(interpret_injury_total_bb2020(7, true, true), Some(InjuryOutcome::Stunned));
    }

    #[test]
    fn bb2020_stunty_thick_skull_at8_thick_skull_does_not_save() {
        // ThickSkull only saves non-Stunty at 8 in BB2020
        assert_eq!(interpret_injury_total_bb2020(8, true, true), Some(InjuryOutcome::KnockedOut));
    }

    #[test]
    fn bb2020_stunty_at9_becomes_badly_hurt() {
        assert_eq!(interpret_injury_total_bb2020(9, true, false), Some(InjuryOutcome::BadlyHurt));
    }

    // ── casualty tier functions (names mirror CasualtyCalcTest) ───────────────

    #[test]
    fn bb2016_first_die1to3_is_badly_hurt() {
        for die in [1, 2, 3] {
            assert_eq!(casualty_tier_bb2016(die), CasualtyTier::BadlyHurt, "die={die}");
        }
    }

    #[test]
    fn bb2016_first_die4or5_is_serious_injury() {
        for die in [4, 5] {
            assert_eq!(casualty_tier_bb2016(die), CasualtyTier::SeriousInjury, "die={die}");
        }
    }

    #[test]
    fn bb2016_first_die6_is_rip() {
        assert_eq!(casualty_tier_bb2016(6), CasualtyTier::Dead);
    }

    #[test]
    fn bb2016_requires_si_roll_only_for4and5() {
        for die in [4, 5] {
            assert!(requires_si_table_bb2016(die), "die={die}");
        }
    }

    #[test]
    fn bb2016_no_si_roll_for1to3and6() {
        for die in [1, 2, 3, 6] {
            assert!(!requires_si_table_bb2016(die), "die={die}");
        }
    }

    #[test]
    fn bb2020_roll1to6_is_badly_hurt() {
        for roll in [1, 2, 3, 4, 5, 6] {
            assert_eq!(casualty_tier_bb2020(roll), CasualtyTier::BadlyHurt, "roll={roll}");
        }
    }

    #[test]
    fn bb2020_roll7to14_is_serious_injury() {
        for roll in [7, 8, 9, 10, 11, 12, 13, 14] {
            assert_eq!(casualty_tier_bb2020(roll), CasualtyTier::SeriousInjury, "roll={roll}");
        }
    }

    #[test]
    fn bb2020_roll15plus_is_rip() {
        for roll in [15, 16, 17] {
            assert_eq!(casualty_tier_bb2020(roll), CasualtyTier::Dead, "roll={roll}");
        }
    }

    #[test]
    fn bb2020_requires_si_roll_only_for13and14() {
        assert!(!requires_si_table_bb2020(12));
        assert!(requires_si_table_bb2020(13));
        assert!(requires_si_table_bb2020(14));
        assert!(!requires_si_table_bb2020(15));
    }

    #[test]
    fn bb2020_serious_injury_sub_type_seriously_hurt() {
        assert_eq!(si_sub_type_bb2020(7), Some(SiSubType::SeriouslyHurt));
        assert_eq!(si_sub_type_bb2020(9), Some(SiSubType::SeriouslyHurt));
    }

    #[test]
    fn bb2020_serious_injury_sub_type_serious_injury() {
        assert_eq!(si_sub_type_bb2020(10), Some(SiSubType::SeriousInjury));
        assert_eq!(si_sub_type_bb2020(12), Some(SiSubType::SeriousInjury));
    }

    #[test]
    fn bb2020_serious_injury_sub_type_null_for_si_table_rolls() {
        assert_eq!(si_sub_type_bb2020(13), None);
        assert_eq!(si_sub_type_bb2020(14), None);
    }

    #[test]
    fn bb2020_serious_injury_sub_type_null_below_si_range() {
        assert_eq!(si_sub_type_bb2020(6), None);
    }

    #[test]
    fn bb2025_roll1to8_is_badly_hurt() {
        for roll in [1, 2, 3, 4, 5, 6, 7, 8] {
            assert_eq!(casualty_tier_bb2025(roll), CasualtyTier::BadlyHurt, "roll={roll}");
        }
    }

    #[test]
    fn bb2025_roll9to14_is_serious_injury() {
        for roll in [9, 10, 11, 12, 13, 14] {
            assert_eq!(casualty_tier_bb2025(roll), CasualtyTier::SeriousInjury, "roll={roll}");
        }
    }

    #[test]
    fn bb2025_roll15plus_is_rip() {
        for roll in [15, 16] {
            assert_eq!(casualty_tier_bb2025(roll), CasualtyTier::Dead, "roll={roll}");
        }
    }

    #[test]
    fn bb2025_serious_injury_sub_type_seriously_hurt() {
        assert_eq!(si_sub_type_bb2025(9), Some(SiSubType::SeriouslyHurt));
        assert_eq!(si_sub_type_bb2025(10), Some(SiSubType::SeriouslyHurt));
    }

    #[test]
    fn bb2025_serious_injury_sub_type_serious_injury() {
        assert_eq!(si_sub_type_bb2025(11), Some(SiSubType::SeriousInjury));
        assert_eq!(si_sub_type_bb2025(12), Some(SiSubType::SeriousInjury));
    }

    #[test]
    fn bb2025_serious_injury_sub_type_null_for_si_table_rolls() {
        assert_eq!(si_sub_type_bb2025(13), None);
        assert_eq!(si_sub_type_bb2025(14), None);
    }

    #[test]
    fn bb2025_serious_injury_sub_type_null_below_si_range() {
        assert_eq!(si_sub_type_bb2025(8), None);
    }

    // ── Cross-edition comparison (names mirror CasualtyCalcTest) ──────────────

    #[test]
    fn bb2025_has_higher_badly_hurt_threshold_than_bb2020() {
        // BB2020: roll 7 → SI; BB2025: roll 7 → Badly Hurt
        assert_eq!(casualty_tier_bb2020(7), CasualtyTier::SeriousInjury);
        assert_eq!(casualty_tier_bb2025(7), CasualtyTier::BadlyHurt);
    }

    #[test]
    fn bb2025_roll8_is_badly_hurt_unlike_bb2020() {
        assert_eq!(casualty_tier_bb2020(8), CasualtyTier::SeriousInjury);
        assert_eq!(casualty_tier_bb2025(8), CasualtyTier::BadlyHurt);
    }

    // ── BB2016 SI detail table ────────────────────────────────────────────────

    #[test]
    fn bb2016_si_table_die4_all_mng() {
        let mng_variants = [
            SeriousInjuryKind::BrokenRibs,
            SeriousInjuryKind::Groin,
            SeriousInjuryKind::BrokenJaw,
            SeriousInjuryKind::BrokenJaw,
            SeriousInjuryKind::BrokenShoulder,
            SeriousInjuryKind::SmashedElbow,
            SeriousInjuryKind::ShatteredWrist,
            SeriousInjuryKind::PinchedNerve,
        ];
        for (die2, expected) in (1..=8).zip(mng_variants.iter()) {
            let result = serious_injury_bb2016(4, die2);
            assert_eq!(result, Some(*expected), "die2={die2}");
        }
    }

    #[test]
    fn bb2016_si_table_die5_stat_effects() {
        assert_eq!(serious_injury_bb2016(5, 1), Some(SeriousInjuryKind::SeriousInjuryNi)); // NI
        assert_eq!(serious_injury_bb2016(5, 2), Some(SeriousInjuryKind::SmashedKneeB2016)); // NI (Smashed Knee BB2016)
        assert_eq!(serious_injury_bb2016(5, 5), Some(SeriousInjuryKind::HeadInjuryAv)); // -AV
        assert_eq!(serious_injury_bb2016(5, 7), Some(SeriousInjuryKind::NeckInjuryAg)); // -AG
        assert_eq!(serious_injury_bb2016(5, 8), Some(SeriousInjuryKind::BrokenCollarBone)); // -ST
    }

    #[test]
    fn bb2016_si_table_out_of_range_returns_none() {
        assert_eq!(serious_injury_bb2016(1, 1), None);
        assert_eq!(serious_injury_bb2016(3, 6), None);
        assert_eq!(serious_injury_bb2016(6, 1), None);
    }

    // ── BB2020/BB2025 SI detail table ─────────────────────────────────────────

    #[test]
    fn bb2020_si_table_all_six_rolls() {
        assert_eq!(serious_injury_bb2020(1), Some(SeriousInjuryKind::HeadInjuryAv));
        assert_eq!(serious_injury_bb2020(2), Some(SeriousInjuryKind::HeadInjuryAv));
        assert_eq!(serious_injury_bb2020(3), Some(SeriousInjuryKind::SmashedKneeMa));
        assert_eq!(serious_injury_bb2020(4), Some(SeriousInjuryKind::BrokenArmPa));
        assert_eq!(serious_injury_bb2020(5), Some(SeriousInjuryKind::NeckInjuryAg));
        assert_eq!(serious_injury_bb2020(6), Some(SeriousInjuryKind::DislocatedShoulderSt));
    }

    #[test]
    fn bb2025_si_table_roll5_is_dislocated_hip_ag() {
        // BB2025 roll 5 = DISLOCATED_HIP (-AG) — now its own variant
        assert_eq!(serious_injury_bb2025(5), Some(SeriousInjuryKind::DislocatedHipAg));
        // roll 6 = DISLOCATED_SHOULDER (-ST), same as BB2020
        assert_eq!(serious_injury_bb2025(6), Some(SeriousInjuryKind::DislocatedShoulderSt));
    }

    #[test]
    fn bb2025_si_table_differs_from_bb2020_at_roll5() {
        assert_eq!(serious_injury_bb2020(5), Some(SeriousInjuryKind::NeckInjuryAg));
        assert_eq!(serious_injury_bb2025(5), Some(SeriousInjuryKind::DislocatedHipAg));
    }

    // ── Remaining InjuryCalcTest mirrors ──────────────────────────────────────

    #[test]
    fn bb2016_thick_skull_at9_still_ko() {
        // ThickSkull only saves at exactly 8 in BB2016; 9 is still KO
        assert_eq!(interpret_injury_total_bb2016(9, false, true), Some(InjuryOutcome::KnockedOut));
    }

    #[test]
    fn bb2020_stunty_at10_is_casualty() {
        // Even with Stunty the casualty threshold is still 10+
        assert_eq!(interpret_injury_total_bb2020(10, true, false), None);
    }
}
