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
            SeriousInjuryKind::HeadInjuryAv => Some(InjuryAttribute::AV),
            SeriousInjuryKind::SmashedKneeMa | SeriousInjuryKind::SmashedKneeB2016 => {
                Some(InjuryAttribute::MA)
            }
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
            SeriousInjuryKind::SmashedHip       // BB2016: -MA (Java: SMASHED_HIP → InjuryAttribute.MA)
            | SeriousInjuryKind::SmashedAnkle
            | SeriousInjuryKind::ThighStrain
            | SeriousInjuryKind::ThumbSprain => Some(InjuryAttribute::MA),
            SeriousInjuryKind::PinchedNerve => Some(InjuryAttribute::AV),
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
    fn bb2016_smashed_knee_reduces_ma() {
        assert_eq!(SeriousInjuryKind::SmashedKneeB2016.injury_attribute(), Some(InjuryAttribute::MA));
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
