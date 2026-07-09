/// 1:1 translation of com.fumbbl.ffb.injury.CrowdPushForSpp.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct CrowdPushForSpp {
    base: InjuryType,
}

impl CrowdPushForSpp {
    pub fn new() -> Self {
        Self { base: InjuryType::new("crowdpushForSpp", true, SendToBoxReason::CROWD_PUSHED) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn can_apo_ko_into_stun(&self) -> bool { false }

    pub fn falling_down_causes_turnover(&self) -> bool { false }

    pub fn is_caused_by_opponent(&self) -> bool { true }
}

impl Default for CrowdPushForSpp {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(CrowdPushForSpp::new().base().name(), "crowdpushForSpp");
    }

    #[test]
    fn worth_spps_is_true() {
        assert!(CrowdPushForSpp::new().base().is_worth_spps());
    }

    #[test]
    fn can_apo_ko_into_stun_is_false() {
        assert!(!CrowdPushForSpp::new().can_apo_ko_into_stun());
    }

    #[test]
    fn falling_down_causes_turnover_is_false() {
        assert!(!CrowdPushForSpp::new().falling_down_causes_turnover());
    }

    #[test]
    fn is_caused_by_opponent_is_true() {
        assert!(CrowdPushForSpp::new().is_caused_by_opponent());
    }
}
