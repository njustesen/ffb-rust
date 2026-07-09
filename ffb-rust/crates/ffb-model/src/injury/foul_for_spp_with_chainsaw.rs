/// 1:1 translation of com.fumbbl.ffb.injury.FoulForSppWithChainsaw.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct FoulForSppWithChainsaw {
    base: InjuryType,
}

impl FoulForSppWithChainsaw {
    pub fn new() -> Self {
        Self { base: InjuryType::new("foulForSppWithChainsaw", true, SendToBoxReason::FOULED) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn should_play_fall_sound(&self) -> bool { false }

    pub fn is_foul(&self) -> bool { true }

    pub fn is_caused_by_opponent(&self) -> bool { true }

    pub fn is_chainsaw(&self) -> bool { true }
}

impl Default for FoulForSppWithChainsaw {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(FoulForSppWithChainsaw::new().base().name(), "foulForSppWithChainsaw");
    }

    #[test]
    fn worth_spps_is_true() {
        assert!(FoulForSppWithChainsaw::new().base().is_worth_spps());
    }

    #[test]
    fn is_foul_is_true() {
        assert!(FoulForSppWithChainsaw::new().is_foul());
    }

    #[test]
    fn is_chainsaw_is_true() {
        assert!(FoulForSppWithChainsaw::new().is_chainsaw());
    }

    #[test]
    fn is_caused_by_opponent_is_true() {
        assert!(FoulForSppWithChainsaw::new().is_caused_by_opponent());
    }
}
