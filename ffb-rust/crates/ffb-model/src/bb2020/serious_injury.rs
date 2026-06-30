use crate::model::injury_attribute::InjuryAttribute;
use crate::model::serious_injury::SeriousInjury as ISeriousInjury;

/// 1:1 translation of com.fumbbl.ffb.bb2020.SeriousInjury.
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SeriousInjury {
    SERIOUSLY_HURT,
    SERIOUS_INJURY,
    HEAD_INJURY,
    SMASHED_KNEE,
    BROKEN_ARM,
    NECK_INJURY,
    DISLOCATED_SHOULDER,
    DEAD,
}

impl ISeriousInjury for SeriousInjury {
    fn get_name(&self) -> &str {
        match self {
            SeriousInjury::SERIOUSLY_HURT => "Seriously Hurt (MNG)",
            SeriousInjury::SERIOUS_INJURY => "Serious Injury (NI)",
            SeriousInjury::HEAD_INJURY => "Head Injury (-AV)",
            SeriousInjury::SMASHED_KNEE => "Smashed Knee (-MA)",
            SeriousInjury::BROKEN_ARM => "Broken Arm (-PA)",
            SeriousInjury::NECK_INJURY => "Neck Injury (-AG)",
            SeriousInjury::DISLOCATED_SHOULDER => "Dislocated Shoulder (-ST)",
            SeriousInjury::DEAD => "Dead (RIP)",
        }
    }

    fn get_button_text(&self) -> &str {
        match self {
            SeriousInjury::SERIOUSLY_HURT => "Seriously Hurt (Miss next game)",
            SeriousInjury::SERIOUS_INJURY => "Serious Injury (Niggling Injury)",
            SeriousInjury::HEAD_INJURY => "Head Injury (-1 AV)",
            SeriousInjury::SMASHED_KNEE => "Smashed Knee (-1 MA)",
            SeriousInjury::BROKEN_ARM => "Broken Arm (-1 PA)",
            SeriousInjury::NECK_INJURY => "Neck Injury (-1 AG)",
            SeriousInjury::DISLOCATED_SHOULDER => "Dislocated Shoulder (-1 ST)",
            SeriousInjury::DEAD => "Dead (RIP)",
        }
    }

    fn get_description(&self) -> &str {
        match self {
            SeriousInjury::SERIOUSLY_HURT => "is seriously hurt (Miss next game)",
            SeriousInjury::SERIOUS_INJURY => "is seriously injured (Niggling Injury)",
            SeriousInjury::HEAD_INJURY => "suffered a head injury (-1 AV)",
            SeriousInjury::SMASHED_KNEE => "suffered a smashed knee (-1 MA)",
            SeriousInjury::BROKEN_ARM => "suffered a broken arm (-1 PA)",
            SeriousInjury::NECK_INJURY => "suffered a neck injury (-1 AG)",
            SeriousInjury::DISLOCATED_SHOULDER => "suffered a dislocated shoulder (-1 ST)",
            SeriousInjury::DEAD => "is dead",
        }
    }

    fn get_recovery(&self) -> &str {
        match self {
            SeriousInjury::SERIOUSLY_HURT => "has been seriously hurt in the previous game (Miss Next Game)",
            SeriousInjury::SERIOUS_INJURY => "has been seriously injured in the previous game (Niggling Injury)",
            SeriousInjury::HEAD_INJURY => "is recovering from a head injury (-1 AV)",
            SeriousInjury::SMASHED_KNEE => "is recovering from a smashed knee (-1 MA)",
            SeriousInjury::BROKEN_ARM => "is recovering from a broken arm (-1 PA)",
            SeriousInjury::NECK_INJURY => "is recovering from a neck injury (-1 AG)",
            SeriousInjury::DISLOCATED_SHOULDER => "is recovering from a dislocated shoulder (-1 ST)",
            SeriousInjury::DEAD => "is dead",
        }
    }

    fn get_injury_attribute(&self) -> Option<InjuryAttribute> {
        match self {
            SeriousInjury::SERIOUS_INJURY => Some(InjuryAttribute::NI),
            SeriousInjury::HEAD_INJURY => Some(InjuryAttribute::AV),
            SeriousInjury::SMASHED_KNEE => Some(InjuryAttribute::MA),
            SeriousInjury::BROKEN_ARM => Some(InjuryAttribute::PA),
            SeriousInjury::NECK_INJURY => Some(InjuryAttribute::AG),
            SeriousInjury::DISLOCATED_SHOULDER => Some(InjuryAttribute::ST),
            _ => None,
        }
    }

    fn is_dead(&self) -> bool {
        *self == SeriousInjury::DEAD
    }

    fn is_poison(&self) -> bool {
        false
    }

    fn show_si_roll(&self) -> bool {
        matches!(
            self,
            SeriousInjury::HEAD_INJURY
                | SeriousInjury::SMASHED_KNEE
                | SeriousInjury::BROKEN_ARM
                | SeriousInjury::NECK_INJURY
                | SeriousInjury::DISLOCATED_SHOULDER
        )
    }
}
