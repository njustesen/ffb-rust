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
