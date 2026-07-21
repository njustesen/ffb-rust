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
