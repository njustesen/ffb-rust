/// 1:1 translation of com.fumbbl.ffb.injury.CrowdPush.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct CrowdPush {
    base: InjuryType,
}

impl CrowdPush {
    pub fn new() -> Self {
        Self { base: InjuryType::new("crowdpush", false, SendToBoxReason::CROWD_PUSHED) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn can_apo_ko_into_stun(&self) -> bool { false }

    pub fn falling_down_causes_turnover(&self) -> bool { false }
}

impl Default for CrowdPush {
    fn default() -> Self { Self::new() }
}
