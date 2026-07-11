use serde::{Deserialize, Serialize};

pub use crate::model::injury_attribute::InjuryAttribute;

/// Serious injury outcomes — shared across BB2016/BB2020/BB2025.
/// BB2016 adds a different set vs BB2020; encoded here as a single enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SeriousInjuryKind {
    // BB2020/BB2025 injuries
    SeriouslyHurt,
    SeriousInjuryNi,
    HeadInjuryAv,
    SmashedKneeMa,
    BrokenArmPa,
    NeckInjuryAg,
    DislocatedHipAg,
    DislocatedShoulderSt,
    Dead,

    // BB2016-only injuries (different table mapping)
    BrokenRibs,
    Groin,
    GougedEye,
    BrokenJaw,
    FracturedArm,
    FracturedLeg,
    SmashedHand,
    PinchedNerve,
    DamagedBack,
    SmashedKneeB2016,
    SmashedHip,
    SmashedAnkle,
    SeriousConcussion,
    FracturedSkull,
    BrokenNeck,
    BrokenCollarBone,
    Poisoned,
    ThighStrain,
    ThumbSprain,
    BrokenShoulder,
    ShatteredWrist,
    SmashedElbow,
    BrokenNose,
}

impl SeriousInjuryKind {
    /// Java: enum constant `.name()` used as the wire string by
    /// `JsonEnumWithNameOption` (`IJsonOption.SERIOUS_INJURY`). BB2020/BB2025's
    /// `SeriousInjury.java` and BB2016's `SeriousInjury.java` are separate Java
    /// enums; this consolidated Rust enum maps each variant to its source
    /// constant name. `SmashedKneeMa` (bb2020/bb2025 `SMASHED_KNEE`) and
    /// `SmashedKneeB2016` (bb2016 `SMASHED_KNEE`) share Java's declared text —
    /// only the enclosing (edition-specific) Java type disambiguates them, so
    /// `from_name` here resolves to `SmashedKneeMa` for that string.
    /// `ThighStrain`, `ThumbSprain`, `BrokenShoulder`, `ShatteredWrist`,
    /// `SmashedElbow`, `BrokenNose` have no confirmed Java source constant
    /// (not found in `bb2016/SeriousInjury.java`); their wire strings are a
    /// best-effort SCREAMING_SNAKE_CASE of the Rust variant name pending
    /// verification against the actual Java source.
    pub fn name(self) -> &'static str {
        match self {
            SeriousInjuryKind::SeriouslyHurt => "SERIOUSLY_HURT",
            SeriousInjuryKind::SeriousInjuryNi => "SERIOUS_INJURY",
            SeriousInjuryKind::HeadInjuryAv => "HEAD_INJURY",
            SeriousInjuryKind::SmashedKneeMa => "SMASHED_KNEE",
            SeriousInjuryKind::BrokenArmPa => "BROKEN_ARM",
            SeriousInjuryKind::NeckInjuryAg => "NECK_INJURY",
            SeriousInjuryKind::DislocatedHipAg => "DISLOCATED_HIP",
            SeriousInjuryKind::DislocatedShoulderSt => "DISLOCATED_SHOULDER",
            SeriousInjuryKind::Dead => "DEAD",
            SeriousInjuryKind::BrokenRibs => "BROKEN_RIBS",
            SeriousInjuryKind::Groin => "GROIN_STRAIN",
            SeriousInjuryKind::GougedEye => "GOUGED_EYE",
            SeriousInjuryKind::BrokenJaw => "BROKEN_JAW",
            SeriousInjuryKind::FracturedArm => "FRACTURED_ARM",
            SeriousInjuryKind::FracturedLeg => "FRACTURED_LEG",
            SeriousInjuryKind::SmashedHand => "SMASHED_HAND",
            SeriousInjuryKind::PinchedNerve => "PINCHED_NERVE",
            SeriousInjuryKind::DamagedBack => "DAMAGED_BACK",
            SeriousInjuryKind::SmashedKneeB2016 => "SMASHED_KNEE",
            SeriousInjuryKind::SmashedHip => "SMASHED_HIP",
            SeriousInjuryKind::SmashedAnkle => "SMASHED_ANKLE",
            SeriousInjuryKind::SeriousConcussion => "SERIOUS_CONCUSSION",
            SeriousInjuryKind::FracturedSkull => "FRACTURED_SKULL",
            SeriousInjuryKind::BrokenNeck => "BROKEN_NECK",
            SeriousInjuryKind::BrokenCollarBone => "SMASHED_COLLAR_BONE",
            SeriousInjuryKind::Poisoned => "POISONED",
            SeriousInjuryKind::ThighStrain => "THIGH_STRAIN",
            SeriousInjuryKind::ThumbSprain => "THUMB_SPRAIN",
            SeriousInjuryKind::BrokenShoulder => "BROKEN_SHOULDER",
            SeriousInjuryKind::ShatteredWrist => "SHATTERED_WRIST",
            SeriousInjuryKind::SmashedElbow => "SMASHED_ELBOW",
            SeriousInjuryKind::BrokenNose => "BROKEN_NOSE",
        }
    }

    /// Java: `SeriousInjury.valueOf(name)` via the appropriate edition factory.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "SERIOUSLY_HURT" => Some(SeriousInjuryKind::SeriouslyHurt),
            "SERIOUS_INJURY" => Some(SeriousInjuryKind::SeriousInjuryNi),
            "HEAD_INJURY" => Some(SeriousInjuryKind::HeadInjuryAv),
            "SMASHED_KNEE" => Some(SeriousInjuryKind::SmashedKneeMa),
            "BROKEN_ARM" => Some(SeriousInjuryKind::BrokenArmPa),
            "NECK_INJURY" => Some(SeriousInjuryKind::NeckInjuryAg),
            "DISLOCATED_HIP" => Some(SeriousInjuryKind::DislocatedHipAg),
            "DISLOCATED_SHOULDER" => Some(SeriousInjuryKind::DislocatedShoulderSt),
            "DEAD" => Some(SeriousInjuryKind::Dead),
            "BROKEN_RIBS" => Some(SeriousInjuryKind::BrokenRibs),
            "GROIN_STRAIN" => Some(SeriousInjuryKind::Groin),
            "GOUGED_EYE" => Some(SeriousInjuryKind::GougedEye),
            "BROKEN_JAW" => Some(SeriousInjuryKind::BrokenJaw),
            "FRACTURED_ARM" => Some(SeriousInjuryKind::FracturedArm),
            "FRACTURED_LEG" => Some(SeriousInjuryKind::FracturedLeg),
            "SMASHED_HAND" => Some(SeriousInjuryKind::SmashedHand),
            "PINCHED_NERVE" => Some(SeriousInjuryKind::PinchedNerve),
            "DAMAGED_BACK" => Some(SeriousInjuryKind::DamagedBack),
            "SMASHED_HIP" => Some(SeriousInjuryKind::SmashedHip),
            "SMASHED_ANKLE" => Some(SeriousInjuryKind::SmashedAnkle),
            "SERIOUS_CONCUSSION" => Some(SeriousInjuryKind::SeriousConcussion),
            "FRACTURED_SKULL" => Some(SeriousInjuryKind::FracturedSkull),
            "BROKEN_NECK" => Some(SeriousInjuryKind::BrokenNeck),
            "SMASHED_COLLAR_BONE" => Some(SeriousInjuryKind::BrokenCollarBone),
            "POISONED" => Some(SeriousInjuryKind::Poisoned),
            "THIGH_STRAIN" => Some(SeriousInjuryKind::ThighStrain),
            "THUMB_SPRAIN" => Some(SeriousInjuryKind::ThumbSprain),
            "BROKEN_SHOULDER" => Some(SeriousInjuryKind::BrokenShoulder),
            "SHATTERED_WRIST" => Some(SeriousInjuryKind::ShatteredWrist),
            "SMASHED_ELBOW" => Some(SeriousInjuryKind::SmashedElbow),
            "BROKEN_NOSE" => Some(SeriousInjuryKind::BrokenNose),
            _ => None,
        }
    }

    pub fn is_dead(self) -> bool {
        self == SeriousInjuryKind::Dead
    }

    pub fn injury_attribute(self) -> Option<InjuryAttribute> {
        match self {
            SeriousInjuryKind::HeadInjuryAv => Some(InjuryAttribute::AV),
            SeriousInjuryKind::SmashedKneeMa => Some(InjuryAttribute::MA),
            SeriousInjuryKind::SmashedKneeB2016 => Some(InjuryAttribute::NI),
            SeriousInjuryKind::BrokenArmPa => Some(InjuryAttribute::PA),
            SeriousInjuryKind::NeckInjuryAg
            | SeriousInjuryKind::DislocatedHipAg
            | SeriousInjuryKind::BrokenNeck => Some(InjuryAttribute::AG),
            SeriousInjuryKind::DislocatedShoulderSt => Some(InjuryAttribute::ST),
            SeriousInjuryKind::SeriousInjuryNi => Some(InjuryAttribute::NI),
            SeriousInjuryKind::GougedEye => Some(InjuryAttribute::AG),
            SeriousInjuryKind::BrokenCollarBone
            | SeriousInjuryKind::SmashedElbow
            | SeriousInjuryKind::ShatteredWrist
            | SeriousInjuryKind::BrokenShoulder => Some(InjuryAttribute::ST),
            SeriousInjuryKind::SmashedHip
            | SeriousInjuryKind::SmashedAnkle
            | SeriousInjuryKind::ThighStrain
            | SeriousInjuryKind::ThumbSprain => Some(InjuryAttribute::MA),
            SeriousInjuryKind::SeriousConcussion | SeriousInjuryKind::FracturedSkull => {
                Some(InjuryAttribute::AV)
            }
            SeriousInjuryKind::DamagedBack => Some(InjuryAttribute::NI),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn injury_attribute_round_trip() {
        let attrs = [
            InjuryAttribute::MA,
            InjuryAttribute::ST,
            InjuryAttribute::AG,
            InjuryAttribute::AV,
            InjuryAttribute::NI,
            InjuryAttribute::PA,
        ];
        for a in &attrs {
            assert_eq!(InjuryAttribute::for_name(a.get_name()), Some(*a));
        }
    }

    #[test]
    fn injury_attribute_strips_sign() {
        assert_eq!(InjuryAttribute::for_name("-MA"), Some(InjuryAttribute::MA));
        assert_eq!(InjuryAttribute::for_name("+AV"), Some(InjuryAttribute::AV));
    }

    #[test]
    fn dead_is_dead() {
        assert!(SeriousInjuryKind::Dead.is_dead());
        assert!(!SeriousInjuryKind::SeriouslyHurt.is_dead());
    }

    #[test]
    fn bb2020_head_injury_reduces_av() {
        assert_eq!(SeriousInjuryKind::HeadInjuryAv.injury_attribute(), Some(InjuryAttribute::AV));
    }

    #[test]
    fn bb2020_smashed_knee_reduces_ma() {
        assert_eq!(SeriousInjuryKind::SmashedKneeMa.injury_attribute(), Some(InjuryAttribute::MA));
    }

    #[test]
    fn bb2020_broken_arm_reduces_pa() {
        assert_eq!(SeriousInjuryKind::BrokenArmPa.injury_attribute(), Some(InjuryAttribute::PA));
    }

    #[test]
    fn bb2020_neck_injury_reduces_ag() {
        assert_eq!(SeriousInjuryKind::NeckInjuryAg.injury_attribute(), Some(InjuryAttribute::AG));
    }

    #[test]
    fn bb2020_dislocated_hip_reduces_ag() {
        assert_eq!(SeriousInjuryKind::DislocatedHipAg.injury_attribute(), Some(InjuryAttribute::AG));
    }

    #[test]
    fn bb2020_dislocated_shoulder_reduces_st() {
        assert_eq!(SeriousInjuryKind::DislocatedShoulderSt.injury_attribute(), Some(InjuryAttribute::ST));
    }

    #[test]
    fn bb2016_broken_ribs_has_no_attribute() {
        assert_eq!(SeriousInjuryKind::BrokenRibs.injury_attribute(), None);
    }

    #[test]
    fn bb2016_smashed_knee_is_niggling() {
        assert_eq!(SeriousInjuryKind::SmashedKneeB2016.injury_attribute(), Some(InjuryAttribute::NI));
    }

    #[test]
    fn bb2016_broken_neck_reduces_ag() {
        assert_eq!(SeriousInjuryKind::BrokenNeck.injury_attribute(), Some(InjuryAttribute::AG));
    }

    #[test]
    fn bb2016_broken_collar_bone_reduces_st() {
        assert_eq!(SeriousInjuryKind::BrokenCollarBone.injury_attribute(), Some(InjuryAttribute::ST));
    }

    #[test]
    fn bb2016_smashed_hip_reduces_ma() {
        assert_eq!(SeriousInjuryKind::SmashedHip.injury_attribute(), Some(InjuryAttribute::MA));
    }

    #[test]
    fn bb2016_seriously_hurt_has_no_attribute() {
        assert_eq!(SeriousInjuryKind::SeriouslyHurt.injury_attribute(), None);
    }
}
