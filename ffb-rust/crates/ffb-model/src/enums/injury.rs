use serde::{Deserialize, Serialize};

/// Which player attribute is reduced by a serious injury.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InjuryAttribute {
    Ma,
    St,
    Ag,
    Av,
    Ni,
    Pa,
}

impl InjuryAttribute {
    pub fn id(self) -> u8 {
        match self {
            InjuryAttribute::Ma => 1,
            InjuryAttribute::St => 2,
            InjuryAttribute::Ag => 3,
            InjuryAttribute::Av => 4,
            InjuryAttribute::Ni => 5,
            InjuryAttribute::Pa => 6,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            InjuryAttribute::Ma => "MA",
            InjuryAttribute::St => "ST",
            InjuryAttribute::Ag => "AG",
            InjuryAttribute::Av => "AV",
            InjuryAttribute::Ni => "NI",
            InjuryAttribute::Pa => "PA",
        }
    }

    pub fn from_name(name: &str) -> Option<InjuryAttribute> {
        let clean = name.trim_start_matches(['+', '-']);
        match clean.to_uppercase().as_str() {
            "MA" => Some(InjuryAttribute::Ma),
            "ST" => Some(InjuryAttribute::St),
            "AG" => Some(InjuryAttribute::Ag),
            "AV" => Some(InjuryAttribute::Av),
            "NI" => Some(InjuryAttribute::Ni),
            "PA" => Some(InjuryAttribute::Pa),
            _ => None,
        }
    }
}

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
    BrokenNeck,
    SmashedAnkle,
    SmashedHip,
    BrokenCollarBone,
    SmashedKneeB2016,
    ThighStrain,
    ThumbSprain,
    BrokenShoulder,
    ShatteredWrist,
    SmashedElbow,
    PinchedNerve,
    BrokenNose,
}

impl SeriousInjuryKind {
    pub fn is_dead(self) -> bool {
        self == SeriousInjuryKind::Dead
    }

    pub fn injury_attribute(self) -> Option<InjuryAttribute> {
        match self {
            SeriousInjuryKind::HeadInjuryAv => Some(InjuryAttribute::Av),
            SeriousInjuryKind::SmashedKneeMa | SeriousInjuryKind::SmashedKneeB2016 => {
                Some(InjuryAttribute::Ma)
            }
            SeriousInjuryKind::BrokenArmPa => Some(InjuryAttribute::Pa),
            SeriousInjuryKind::NeckInjuryAg
            | SeriousInjuryKind::DislocatedHipAg
            | SeriousInjuryKind::BrokenNeck => Some(InjuryAttribute::Ag),
            SeriousInjuryKind::DislocatedShoulderSt => Some(InjuryAttribute::St),
            SeriousInjuryKind::SeriousInjuryNi => Some(InjuryAttribute::Ni),
            SeriousInjuryKind::GougedEye => Some(InjuryAttribute::Ag),
            SeriousInjuryKind::BrokenCollarBone
            | SeriousInjuryKind::SmashedElbow
            | SeriousInjuryKind::ShatteredWrist
            | SeriousInjuryKind::BrokenShoulder => Some(InjuryAttribute::St),
            SeriousInjuryKind::SmashedHip       // BB2016: -MA (Java: SMASHED_HIP → InjuryAttribute.MA)
            | SeriousInjuryKind::SmashedAnkle
            | SeriousInjuryKind::ThighStrain
            | SeriousInjuryKind::ThumbSprain => Some(InjuryAttribute::Ma),
            SeriousInjuryKind::PinchedNerve => Some(InjuryAttribute::Av),
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
            InjuryAttribute::Ma,
            InjuryAttribute::St,
            InjuryAttribute::Ag,
            InjuryAttribute::Av,
            InjuryAttribute::Ni,
            InjuryAttribute::Pa,
        ];
        for a in &attrs {
            assert_eq!(InjuryAttribute::from_name(a.name()), Some(*a));
        }
    }

    #[test]
    fn injury_attribute_strips_sign() {
        assert_eq!(InjuryAttribute::from_name("-MA"), Some(InjuryAttribute::Ma));
        assert_eq!(InjuryAttribute::from_name("+AV"), Some(InjuryAttribute::Av));
    }

    #[test]
    fn dead_is_dead() {
        assert!(SeriousInjuryKind::Dead.is_dead());
        assert!(!SeriousInjuryKind::SeriouslyHurt.is_dead());
    }

    #[test]
    fn bb2020_head_injury_reduces_av() {
        assert_eq!(SeriousInjuryKind::HeadInjuryAv.injury_attribute(), Some(InjuryAttribute::Av));
    }

    #[test]
    fn bb2020_smashed_knee_reduces_ma() {
        assert_eq!(SeriousInjuryKind::SmashedKneeMa.injury_attribute(), Some(InjuryAttribute::Ma));
    }

    #[test]
    fn bb2020_broken_arm_reduces_pa() {
        assert_eq!(SeriousInjuryKind::BrokenArmPa.injury_attribute(), Some(InjuryAttribute::Pa));
    }

    #[test]
    fn bb2020_neck_injury_reduces_ag() {
        assert_eq!(SeriousInjuryKind::NeckInjuryAg.injury_attribute(), Some(InjuryAttribute::Ag));
    }

    #[test]
    fn bb2020_dislocated_hip_reduces_ag() {
        assert_eq!(SeriousInjuryKind::DislocatedHipAg.injury_attribute(), Some(InjuryAttribute::Ag));
    }

    #[test]
    fn bb2020_dislocated_shoulder_reduces_st() {
        assert_eq!(SeriousInjuryKind::DislocatedShoulderSt.injury_attribute(), Some(InjuryAttribute::St));
    }

    #[test]
    fn bb2016_broken_ribs_has_no_attribute() {
        assert_eq!(SeriousInjuryKind::BrokenRibs.injury_attribute(), None);
    }

    #[test]
    fn bb2016_smashed_knee_reduces_ma() {
        assert_eq!(SeriousInjuryKind::SmashedKneeB2016.injury_attribute(), Some(InjuryAttribute::Ma));
    }

    #[test]
    fn bb2016_broken_neck_reduces_ag() {
        assert_eq!(SeriousInjuryKind::BrokenNeck.injury_attribute(), Some(InjuryAttribute::Ag));
    }

    #[test]
    fn bb2016_broken_collar_bone_reduces_st() {
        assert_eq!(SeriousInjuryKind::BrokenCollarBone.injury_attribute(), Some(InjuryAttribute::St));
    }

    #[test]
    fn bb2016_smashed_hip_reduces_ma() {
        assert_eq!(SeriousInjuryKind::SmashedHip.injury_attribute(), Some(InjuryAttribute::Ma));
    }

    #[test]
    fn bb2016_seriously_hurt_has_no_attribute() {
        assert_eq!(SeriousInjuryKind::SeriouslyHurt.injury_attribute(), None);
    }
}
