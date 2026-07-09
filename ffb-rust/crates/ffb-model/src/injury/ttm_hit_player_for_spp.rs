/// 1:1 translation of com.fumbbl.ffb.injury.TtmHitPlayerForSpp.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct TtmHitPlayerForSpp {
    base: InjuryType,
}

impl TtmHitPlayerForSpp {
    pub fn new() -> Self {
        Self { base: InjuryType::new("ttmHitPlayerForSpp", true, SendToBoxReason::HIT_BY_THROWN_PLAYER) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn is_caused_by_opponent(&self) -> bool { true }
}

impl Default for TtmHitPlayerForSpp {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(TtmHitPlayerForSpp::new().base().name(), "ttmHitPlayerForSpp");
    }

    #[test]
    fn worth_spps_is_true() {
        assert!(TtmHitPlayerForSpp::new().base().is_worth_spps());
    }

    #[test]
    fn is_caused_by_opponent_is_true() {
        assert!(TtmHitPlayerForSpp::new().is_caused_by_opponent());
    }
}
