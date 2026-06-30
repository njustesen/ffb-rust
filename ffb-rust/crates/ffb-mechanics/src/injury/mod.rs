use ffb_model::enums::{Rules, SeriousInjuryKind};

/// Context for what kind of action caused this injury.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InjuryCause {
    Block,
    Foul,
    Chainsaw,
    Stab,
    BallAndChain,
    ProjectileVomit,
    ThrowTeamMate,
    Other,
}

/// Map a 2d6 casualty roll to a `SeriousInjuryKind` for the given rules edition.
///
/// BB2020/BB2025 share the same casualty table (p. 66 of BB2020 rulebook).
/// BB2016 uses a different and larger table.
pub fn casualty_result(rules: Rules, roll: i32) -> Option<SeriousInjuryKind> {
    match rules {
        Rules::Bb2016 => casualty_result_bb2016(roll),
        Rules::Bb2020 | Rules::Bb2025 | Rules::Common => casualty_result_bb2020(roll),
    }
}

/// BB2020/BB2025 casualty table (2d6).
///
///  2–7  → Seriously Hurt   (miss next game, no lasting effect)
///  8–9  → Niggling Injury
/// 10    → Head Injury  (-AV)
/// 11    → Smashed Knee (-MA)
/// 12    → Broken Arm   (-PA)
/// 13    → Neck Injury  (-AG)
/// 14    → Dislocated Shoulder (-ST)
/// 15–16 → Dead
pub fn casualty_result_bb2020(roll: i32) -> Option<SeriousInjuryKind> {
    Some(match roll {
        2..=7 => SeriousInjuryKind::SeriouslyHurt,
        8..=9 => SeriousInjuryKind::SeriousInjuryNi,
        10 => SeriousInjuryKind::HeadInjuryAv,
        11 => SeriousInjuryKind::SmashedKneeMa,
        12 => SeriousInjuryKind::BrokenArmPa,
        13 => SeriousInjuryKind::NeckInjuryAg,
        14 => SeriousInjuryKind::DislocatedShoulderSt,
        15..=16 => SeriousInjuryKind::Dead,
        _ => return None,
    })
}

/// BB2016 casualty table (2d6).
///
/// Roll  2: Broken Ribs (miss next game)
/// Roll  3: Groin Strain (miss next game)
/// Roll  4: Gouged Eye (-AG)
/// Roll  5: Broken Jaw (miss next game)
/// Roll  6: Smashed Hip (-MA)
/// Roll  7: Smashed Knee (-MA)
/// Roll  8: Broken Collarbone (-ST)
/// Roll  9: Serious Concussion (Ni)
/// Roll 10: Fractured Skull (Ni)
/// Roll 11: Broken Neck (-AG)
/// Roll 12: Smashed Shoulder (-ST)
/// Roll 13: Smashed Elbow (-ST)
/// Roll 14: Shattered Wrist (-ST)
/// Roll 15: Smashed Ankle (-MA)
/// Roll 16: Dead
pub fn casualty_result_bb2016(roll: i32) -> Option<SeriousInjuryKind> {
    Some(match roll {
        2 => SeriousInjuryKind::BrokenRibs,
        3 => SeriousInjuryKind::Groin,
        4 => SeriousInjuryKind::GougedEye,
        5 => SeriousInjuryKind::BrokenJaw,
        6 => SeriousInjuryKind::SmashedHip,
        7 => SeriousInjuryKind::SmashedKneeB2016,
        8 => SeriousInjuryKind::BrokenCollarBone,
        9 | 10 => SeriousInjuryKind::SeriousInjuryNi,
        11 => SeriousInjuryKind::BrokenNeck,
        12 => SeriousInjuryKind::BrokenShoulder,
        13 => SeriousInjuryKind::SmashedElbow,
        14 => SeriousInjuryKind::ShatteredWrist,
        15 => SeriousInjuryKind::SmashedAnkle,
        16 => SeriousInjuryKind::Dead,
        _ => return None,
    })
}

/// High-level casualty outcome tier (mirrors Java CasualtyCalc tier return values).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CasualtyTier {
    BadlyHurt,
    SeriousInjury,
    Rip,
}

/// BB2016 casualty tier from the first D6 die (1–6 only).
/// 1–3 = Badly Hurt, 4–5 = Serious Injury (SI sub-table needed), 6 = RIP.
pub fn casualty_tier_bb2016(first_die: i32) -> CasualtyTier {
    match first_die {
        6 => CasualtyTier::Rip,
        4 | 5 => CasualtyTier::SeriousInjury,
        _ => CasualtyTier::BadlyHurt,
    }
}

/// BB2016: whether the result requires an additional SI sub-table roll (only 4 or 5).
pub fn requires_si_roll_bb2016(first_die: i32) -> bool {
    first_die == 4 || first_die == 5
}

/// BB2020 casualty tier from a d16 roll.
/// 1–6 = Badly Hurt, 7–14 = Serious Injury, 15+ = RIP.
pub fn casualty_tier_bb2020(roll: i32) -> CasualtyTier {
    if roll >= 15 { CasualtyTier::Rip } else if roll >= 7 { CasualtyTier::SeriousInjury } else { CasualtyTier::BadlyHurt }
}

/// BB2025 casualty tier from a d16 roll.
/// 1–8 = Badly Hurt, 9–14 = Serious Injury, 15+ = RIP.
pub fn casualty_tier_bb2025(roll: i32) -> CasualtyTier {
    if roll >= 15 { CasualtyTier::Rip } else if roll >= 9 { CasualtyTier::SeriousInjury } else { CasualtyTier::BadlyHurt }
}

/// BB2020: whether the d16 roll triggers the SI detail sub-table (only 13–14).
pub fn requires_si_roll_bb2020(roll: i32) -> bool {
    roll == 13 || roll == 14
}

/// BB2025 d16 casualty roll → SeriousInjuryKind.
/// 1-8 = Badly Hurt (None), 9-14 = Serious Injury, 15-16 = Dead.
pub fn serious_injury_kind_bb2025(roll: i32) -> Option<ffb_model::enums::SeriousInjuryKind> {
    use ffb_model::enums::SeriousInjuryKind as K;
    match roll {
        1..=8   => None,
        9       => Some(K::SmashedKneeMa),
        10      => Some(K::HeadInjuryAv),
        11      => Some(K::BrokenArmPa),
        12      => Some(K::NeckInjuryAg),
        13      => Some(K::DislocatedHipAg),
        14      => Some(K::DislocatedShoulderSt),
        _       => Some(K::Dead),  // 15-16
    }
}

/// BB2020 serious injury sub-type string for rolls 7–12 (None for 13–14 which use SI table).
pub fn serious_injury_sub_type_bb2020(roll: i32) -> Option<&'static str> {
    match roll {
        10..=12 => Some("SERIOUS_INJURY"),
        7..=9 => Some("SERIOUSLY_HURT"),
        _ => None,
    }
}

/// BB2025 serious injury sub-type string for rolls 9–12 (None for 13–14 which use SI table).
pub fn serious_injury_sub_type_bb2025(roll: i32) -> Option<&'static str> {
    match roll {
        11..=12 => Some("SERIOUS_INJURY"),
        9..=10 => Some("SERIOUSLY_HURT"),
        _ => None,
    }
}

/// Whether an apothecary can attempt to save this casualty (not dead, not Stab result).
///
/// In BB2020/BB2025: apothecary can treat any casualty result.
/// In BB2016: same rule applies.
/// Stab results bypass the apothecary in some editions — the `cause` parameter handles this.
pub fn apothecary_can_treat(rules: Rules, cause: InjuryCause, result: SeriousInjuryKind) -> bool {
    if result.is_dead() {
        return false;
    }
    match rules {
        Rules::Bb2016 => cause != InjuryCause::Stab,
        Rules::Bb2020 | Rules::Bb2025 | Rules::Common => {
            cause != InjuryCause::Stab && cause != InjuryCause::Chainsaw
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bb2020_casualty_table_boundaries() {
        assert_eq!(casualty_result(Rules::Bb2020, 2), Some(SeriousInjuryKind::SeriouslyHurt));
        assert_eq!(casualty_result(Rules::Bb2020, 7), Some(SeriousInjuryKind::SeriouslyHurt));
        assert_eq!(casualty_result(Rules::Bb2020, 8), Some(SeriousInjuryKind::SeriousInjuryNi));
        assert_eq!(casualty_result(Rules::Bb2020, 10), Some(SeriousInjuryKind::HeadInjuryAv));
        assert_eq!(casualty_result(Rules::Bb2020, 14), Some(SeriousInjuryKind::DislocatedShoulderSt));
        assert_eq!(casualty_result(Rules::Bb2020, 15), Some(SeriousInjuryKind::Dead));
        assert_eq!(casualty_result(Rules::Bb2020, 16), Some(SeriousInjuryKind::Dead));
    }

    #[test]
    fn bb2016_casualty_dead() {
        assert_eq!(casualty_result(Rules::Bb2016, 16), Some(SeriousInjuryKind::Dead));
    }

    #[test]
    fn bb2016_gouged_eye_is_ag_loss() {
        let result = casualty_result(Rules::Bb2016, 4).unwrap();
        assert_eq!(result, SeriousInjuryKind::GougedEye);
        use ffb_model::enums::InjuryAttribute;
        assert_eq!(result.injury_attribute(), Some(InjuryAttribute::AG));
    }

    #[test]
    fn apo_cannot_treat_dead() {
        assert!(!apothecary_can_treat(Rules::Bb2020, InjuryCause::Block, SeriousInjuryKind::Dead));
    }

    #[test]
    fn apo_cannot_treat_bb2020_chainsaw() {
        assert!(!apothecary_can_treat(Rules::Bb2020, InjuryCause::Chainsaw, SeriousInjuryKind::SeriouslyHurt));
    }

    #[test]
    fn apo_can_treat_normal_block() {
        assert!(apothecary_can_treat(Rules::Bb2020, InjuryCause::Block, SeriousInjuryKind::SeriouslyHurt));
    }

    // ── CasualtyCalcTest parity ───────────────────────────────────────────────

    #[test]
    fn bb2016_first_die_1to3_is_badly_hurt() {
        for die in [1, 2, 3] {
            assert_eq!(casualty_tier_bb2016(die), CasualtyTier::BadlyHurt, "die={die}");
        }
    }

    #[test]
    fn bb2016_first_die_4or5_is_serious_injury() {
        assert_eq!(casualty_tier_bb2016(4), CasualtyTier::SeriousInjury);
        assert_eq!(casualty_tier_bb2016(5), CasualtyTier::SeriousInjury);
    }

    #[test]
    fn bb2016_first_die_6_is_rip() {
        assert_eq!(casualty_tier_bb2016(6), CasualtyTier::Rip);
    }

    #[test]
    fn bb2016_requires_si_roll_only_for_4_and_5() {
        assert!(requires_si_roll_bb2016(4));
        assert!(requires_si_roll_bb2016(5));
    }

    #[test]
    fn bb2016_no_si_roll_for_1to3_and_6() {
        for die in [1, 2, 3, 6] {
            assert!(!requires_si_roll_bb2016(die), "die={die}");
        }
    }

    #[test]
    fn bb2020_roll_1to6_is_badly_hurt() {
        for roll in 1..=6 {
            assert_eq!(casualty_tier_bb2020(roll), CasualtyTier::BadlyHurt, "roll={roll}");
        }
    }

    #[test]
    fn bb2020_roll_7to14_is_serious_injury() {
        for roll in 7..=14 {
            assert_eq!(casualty_tier_bb2020(roll), CasualtyTier::SeriousInjury, "roll={roll}");
        }
    }

    #[test]
    fn bb2020_roll_15plus_is_rip() {
        for roll in [15, 16, 17] {
            assert_eq!(casualty_tier_bb2020(roll), CasualtyTier::Rip, "roll={roll}");
        }
    }

    #[test]
    fn bb2020_requires_si_roll_only_for_13_and_14() {
        assert!(!requires_si_roll_bb2020(12));
        assert!(requires_si_roll_bb2020(13));
        assert!(requires_si_roll_bb2020(14));
        assert!(!requires_si_roll_bb2020(15));
    }

    #[test]
    fn bb2020_serious_injury_sub_type_seriously_hurt() {
        assert_eq!(serious_injury_sub_type_bb2020(7), Some("SERIOUSLY_HURT"));
        assert_eq!(serious_injury_sub_type_bb2020(9), Some("SERIOUSLY_HURT"));
    }

    #[test]
    fn bb2020_serious_injury_sub_type_serious_injury() {
        assert_eq!(serious_injury_sub_type_bb2020(10), Some("SERIOUS_INJURY"));
        assert_eq!(serious_injury_sub_type_bb2020(12), Some("SERIOUS_INJURY"));
    }

    #[test]
    fn bb2020_serious_injury_sub_type_none_for_si_table_rolls() {
        assert_eq!(serious_injury_sub_type_bb2020(13), None);
        assert_eq!(serious_injury_sub_type_bb2020(14), None);
    }

    #[test]
    fn bb2025_roll_1to8_is_badly_hurt() {
        for roll in 1..=8 {
            assert_eq!(casualty_tier_bb2025(roll), CasualtyTier::BadlyHurt, "roll={roll}");
        }
    }

    #[test]
    fn bb2025_roll_9to14_is_serious_injury() {
        for roll in 9..=14 {
            assert_eq!(casualty_tier_bb2025(roll), CasualtyTier::SeriousInjury, "roll={roll}");
        }
    }

    #[test]
    fn bb2025_roll_15plus_is_rip() {
        assert_eq!(casualty_tier_bb2025(15), CasualtyTier::Rip);
        assert_eq!(casualty_tier_bb2025(16), CasualtyTier::Rip);
    }

    #[test]
    fn bb2025_serious_injury_sub_type_seriously_hurt() {
        assert_eq!(serious_injury_sub_type_bb2025(9), Some("SERIOUSLY_HURT"));
        assert_eq!(serious_injury_sub_type_bb2025(10), Some("SERIOUSLY_HURT"));
    }

    #[test]
    fn bb2025_serious_injury_sub_type_serious_injury() {
        assert_eq!(serious_injury_sub_type_bb2025(11), Some("SERIOUS_INJURY"));
        assert_eq!(serious_injury_sub_type_bb2025(12), Some("SERIOUS_INJURY"));
    }

    #[test]
    fn bb2025_has_higher_badly_hurt_threshold_than_bb2020() {
        assert_eq!(casualty_tier_bb2020(7), CasualtyTier::SeriousInjury);
        assert_eq!(casualty_tier_bb2025(7), CasualtyTier::BadlyHurt);
    }

    #[test]
    fn bb2025_roll_8_is_badly_hurt_unlike_bb2020() {
        assert_eq!(casualty_tier_bb2020(8), CasualtyTier::SeriousInjury);
        assert_eq!(casualty_tier_bb2025(8), CasualtyTier::BadlyHurt);
    }
}
