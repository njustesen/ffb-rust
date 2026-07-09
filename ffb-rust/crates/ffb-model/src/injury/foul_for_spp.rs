/// 1:1 translation of com.fumbbl.ffb.injury.FoulForSpp.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct FoulForSpp {
    base: InjuryType,
}

impl FoulForSpp {
    pub fn new() -> Self {
        Self { base: InjuryType::new("foulForSpp", true, SendToBoxReason::FOULED) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn should_play_fall_sound(&self) -> bool { false }

    pub fn is_foul(&self) -> bool { true }

    pub fn is_caused_by_opponent(&self) -> bool { true }
}

impl Default for FoulForSpp {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(FoulForSpp::new().base().name(), "foulForSpp");
    }

    #[test]
    fn worth_spps_is_true() {
        assert!(FoulForSpp::new().base().is_worth_spps());
    }

    #[test]
    fn should_play_fall_sound_is_false() {
        assert!(!FoulForSpp::new().should_play_fall_sound());
    }

    #[test]
    fn is_foul_is_true() {
        assert!(FoulForSpp::new().is_foul());
    }

    #[test]
    fn is_caused_by_opponent_is_true() {
        assert!(FoulForSpp::new().is_caused_by_opponent());
    }
}
