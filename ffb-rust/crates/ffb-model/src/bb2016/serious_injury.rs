use crate::model::injury_attribute::InjuryAttribute;
use crate::model::serious_injury::SeriousInjury as ISeriousInjury;

/// 1:1 translation of com.fumbbl.ffb.bb2016.SeriousInjury.
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SeriousInjury {
    BROKEN_RIBS,
    GROIN_STRAIN,
    GOUGED_EYE,
    BROKEN_JAW,
    FRACTURED_ARM,
    FRACTURED_LEG,
    SMASHED_HAND,
    PINCHED_NERVE,
    DAMAGED_BACK,
    SMASHED_KNEE,
    SMASHED_HIP,
    SMASHED_ANKLE,
    SERIOUS_CONCUSSION,
    FRACTURED_SKULL,
    BROKEN_NECK,
    SMASHED_COLLAR_BONE,
    DEAD,
    POISONED,
}

impl ISeriousInjury for SeriousInjury {
    fn get_name(&self) -> &str {
        match self {
            SeriousInjury::BROKEN_RIBS => "Broken Ribs (MNG)",
            SeriousInjury::GROIN_STRAIN => "Groin Strain (MNG)",
            SeriousInjury::GOUGED_EYE => "Gouged Eye (MNG)",
            SeriousInjury::BROKEN_JAW => "Broken Jaw (MNG)",
            SeriousInjury::FRACTURED_ARM => "Fractured Arm (MNG)",
            SeriousInjury::FRACTURED_LEG => "Fractured Leg (MNG)",
            SeriousInjury::SMASHED_HAND => "Smashed Hand (MNG)",
            SeriousInjury::PINCHED_NERVE => "Pinched Nerve (MNG)",
            SeriousInjury::DAMAGED_BACK => "Damaged Back (NI)",
            SeriousInjury::SMASHED_KNEE => "Smashed Knee (NI)",
            SeriousInjury::SMASHED_HIP => "Smashed Hip (-MA)",
            SeriousInjury::SMASHED_ANKLE => "Smashed Ankle (-MA)",
            SeriousInjury::SERIOUS_CONCUSSION => "Serious Concussion (-AV)",
            SeriousInjury::FRACTURED_SKULL => "Fractured Skull (-AV)",
            SeriousInjury::BROKEN_NECK => "Broken Neck (-AG)",
            SeriousInjury::SMASHED_COLLAR_BONE => "Smashed Collar Bone (-ST)",
            SeriousInjury::DEAD => "Dead (RIP)",
            SeriousInjury::POISONED => "Poisoned (MNG)",
        }
    }

    fn get_button_text(&self) -> &str {
        match self {
            SeriousInjury::BROKEN_RIBS => "Broken Ribs (Miss next game)",
            SeriousInjury::GROIN_STRAIN => "Groin Strain (Miss next game)",
            SeriousInjury::GOUGED_EYE => "Gouged Eye (Miss next game)",
            SeriousInjury::BROKEN_JAW => "Broken Jaw (Miss next game)",
            SeriousInjury::FRACTURED_ARM => "Fractured Arm (Miss next game)",
            SeriousInjury::FRACTURED_LEG => "Fractured Leg (Miss next game)",
            SeriousInjury::SMASHED_HAND => "Smashed Hand (Miss next game)",
            SeriousInjury::PINCHED_NERVE => "Pinched Nerve (Miss next game)",
            SeriousInjury::DAMAGED_BACK => "Damaged Back (Niggling Injury)",
            SeriousInjury::SMASHED_KNEE => "Smashed Knee (Niggling Injury)",
            SeriousInjury::SMASHED_HIP => "Smashed Hip (-1 MA)",
            SeriousInjury::SMASHED_ANKLE => "Smashed Ankle (-1 MA)",
            SeriousInjury::SERIOUS_CONCUSSION => "Serious Concussion (-1 AV)",
            SeriousInjury::FRACTURED_SKULL => "Fractured Skull (-1 AV)",
            SeriousInjury::BROKEN_NECK => "Broken Neck (-1 AG)",
            SeriousInjury::SMASHED_COLLAR_BONE => "Smashed Collar Bone (-1 ST)",
            SeriousInjury::DEAD => "Dead (RIP)",
            SeriousInjury::POISONED => "Poisoned (Miss next game)",
        }
    }

    fn get_description(&self) -> &str {
        match self {
            SeriousInjury::BROKEN_RIBS => "has broken some ribs (Miss next game)",
            SeriousInjury::GROIN_STRAIN => "has got a groin strain (Miss next game)",
            SeriousInjury::GOUGED_EYE => "has got a gouged eye (Miss next game)",
            SeriousInjury::BROKEN_JAW => "has got a broken jaw (Miss next game)",
            SeriousInjury::FRACTURED_ARM => "has got a fractured arm (Miss next game)",
            SeriousInjury::FRACTURED_LEG => "has got a fractured leg (Miss next game)",
            SeriousInjury::SMASHED_HAND => "has got a smashed hand (Miss next game)",
            SeriousInjury::PINCHED_NERVE => "has got a pinched nerve (Miss next game)",
            SeriousInjury::DAMAGED_BACK => "has got a damaged back (Niggling Injury)",
            SeriousInjury::SMASHED_KNEE => "has got a smashed knee (Niggling Injury)",
            SeriousInjury::SMASHED_HIP => "has got a smashed hip (-1 MA)",
            SeriousInjury::SMASHED_ANKLE => "has got a smashed ankle (-1 MA)",
            SeriousInjury::SERIOUS_CONCUSSION => "has got a serious concussion (-1 AV)",
            SeriousInjury::FRACTURED_SKULL => "has got a fractured skull (-1 AV)",
            SeriousInjury::BROKEN_NECK => "has got a broken neck (-1 AG)",
            SeriousInjury::SMASHED_COLLAR_BONE => "has got a smashed collar bone (-1 ST)",
            SeriousInjury::DEAD => "is dead",
            SeriousInjury::POISONED => "has been poisoned (Miss next game)",
        }
    }

    fn get_recovery(&self) -> &str {
        match self {
            SeriousInjury::BROKEN_RIBS => "is recovering from broken ribs",
            SeriousInjury::GROIN_STRAIN => "is recovering from a groin strain",
            SeriousInjury::GOUGED_EYE => "is recovering from a gouged eye",
            SeriousInjury::BROKEN_JAW => "is recovering from a broken jaw",
            SeriousInjury::FRACTURED_ARM => "is recovering from a fractured arm",
            SeriousInjury::FRACTURED_LEG => "is recovering from a fractured leg",
            SeriousInjury::SMASHED_HAND => "is recovering from a smashed hand",
            SeriousInjury::PINCHED_NERVE => "is recovering from a pinched nerve",
            SeriousInjury::DAMAGED_BACK => "is recovering from a damaged back (Niggling Injury)",
            SeriousInjury::SMASHED_KNEE => "is recovering from a smashed knee (Niggling Injury)",
            SeriousInjury::SMASHED_HIP => "is recovering from a smashed hip (-1 MA)",
            SeriousInjury::SMASHED_ANKLE => "is recovering from a smashed ankle (-1 MA)",
            SeriousInjury::SERIOUS_CONCUSSION => "is recovering from a serious concussion (-1 AV)",
            SeriousInjury::FRACTURED_SKULL => "is recovering from a fractured skull (-1 AV)",
            SeriousInjury::BROKEN_NECK => "is recovering from a broken neck (-1 AG)",
            SeriousInjury::SMASHED_COLLAR_BONE => "is recovering from a smashed collar bone (-1 ST)",
            SeriousInjury::DEAD => "is dead",
            SeriousInjury::POISONED => "is recovering from being poisoned",
        }
    }

    fn get_injury_attribute(&self) -> Option<InjuryAttribute> {
        match self {
            SeriousInjury::DAMAGED_BACK | SeriousInjury::SMASHED_KNEE => Some(InjuryAttribute::NI),
            SeriousInjury::SMASHED_HIP | SeriousInjury::SMASHED_ANKLE => Some(InjuryAttribute::MA),
            SeriousInjury::SERIOUS_CONCUSSION | SeriousInjury::FRACTURED_SKULL => Some(InjuryAttribute::AV),
            SeriousInjury::BROKEN_NECK => Some(InjuryAttribute::AG),
            SeriousInjury::SMASHED_COLLAR_BONE => Some(InjuryAttribute::ST),
            _ => None,
        }
    }

    fn is_dead(&self) -> bool {
        *self == SeriousInjury::DEAD
    }

    fn is_poison(&self) -> bool {
        *self == SeriousInjury::POISONED
    }

    fn show_si_roll(&self) -> bool {
        false
    }
}
