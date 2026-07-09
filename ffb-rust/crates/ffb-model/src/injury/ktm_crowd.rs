/// 1:1 translation of com.fumbbl.ffb.injury.KtmCrowd.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct KtmCrowd {
    base: InjuryType,
}

impl KtmCrowd {
    pub fn new() -> Self {
        Self { base: InjuryType::new("ktmCrowd", false, SendToBoxReason::CROWD_KICKED) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }
}

impl Default for KtmCrowd {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(KtmCrowd::new().base().name(), "ktmCrowd");
    }

    #[test]
    fn worth_spps_is_false() {
        assert!(!KtmCrowd::new().base().is_worth_spps());
    }

    #[test]
    fn send_to_box_reason_is_crowd_kicked() {
        use crate::model::send_to_box_reason::SendToBoxReason;
        assert_eq!(KtmCrowd::new().base().send_to_box_reason(), SendToBoxReason::CROWD_KICKED);
    }
}
