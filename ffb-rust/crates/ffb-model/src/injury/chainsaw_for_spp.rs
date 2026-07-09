/// 1:1 translation of com.fumbbl.ffb.injury.ChainsawForSpp.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct ChainsawForSpp {
    base: InjuryType,
}

impl ChainsawForSpp {
    pub fn new() -> Self {
        Self { base: InjuryType::new("chainsawForSpp", true, SendToBoxReason::CHAINSAW) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn is_caused_by_opponent(&self) -> bool { true }

    pub fn is_chainsaw(&self) -> bool { true }
}

impl Default for ChainsawForSpp {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(ChainsawForSpp::new().base().name(), "chainsawForSpp");
    }

    #[test]
    fn worth_spps_is_true() {
        assert!(ChainsawForSpp::new().base().is_worth_spps());
    }

    #[test]
    fn is_caused_by_opponent_is_true() {
        assert!(ChainsawForSpp::new().is_caused_by_opponent());
    }

    #[test]
    fn is_chainsaw_is_true() {
        assert!(ChainsawForSpp::new().is_chainsaw());
    }
}
