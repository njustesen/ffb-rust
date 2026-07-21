/// 1:1 translation of com.fumbbl.ffb.injury.KtmInjury.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct KtmInjury {
    base: InjuryType,
}

impl KtmInjury {
    pub fn new() -> Self {
        Self { base: InjuryType::new("ktmInjury", false, SendToBoxReason::KICKED) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }
}

impl Default for KtmInjury {
    fn default() -> Self { Self::new() }
}
