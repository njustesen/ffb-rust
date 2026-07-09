/// 1:1 translation of com.fumbbl.ffb.injury.Foul.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct Foul {
    base: InjuryType,
}

impl Foul {
    pub fn new() -> Self {
        Self { base: InjuryType::new("foul", false, SendToBoxReason::FOULED) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn should_play_fall_sound(&self) -> bool { false }

    pub fn is_foul(&self) -> bool { true }
}

impl Default for Foul {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(Foul::new().base().name(), "foul");
    }

    #[test]
    fn worth_spps_is_false() {
        assert!(!Foul::new().base().is_worth_spps());
    }

    #[test]
    fn should_play_fall_sound_is_false() {
        assert!(!Foul::new().should_play_fall_sound());
    }

    #[test]
    fn is_foul_is_true() {
        assert!(Foul::new().is_foul());
    }
}
