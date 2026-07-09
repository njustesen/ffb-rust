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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(CrowdPush::new().base().name(), "crowdpush");
    }

    #[test]
    fn worth_spps_is_false() {
        assert!(!CrowdPush::new().base().is_worth_spps());
    }

    #[test]
    fn can_apo_ko_into_stun_is_false() {
        assert!(!CrowdPush::new().can_apo_ko_into_stun());
    }

    #[test]
    fn falling_down_causes_turnover_is_false() {
        assert!(!CrowdPush::new().falling_down_causes_turnover());
    }
}
