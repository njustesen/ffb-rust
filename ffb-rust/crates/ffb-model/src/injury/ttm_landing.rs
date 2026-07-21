/// 1:1 translation of com.fumbbl.ffb.injury.TtmLanding.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct TtmLanding {
    base: InjuryType,
}

impl TtmLanding {
    pub fn new() -> Self {
        Self { base: InjuryType::new("ttmLanding", false, SendToBoxReason::LANDING_FAIL) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }
}

impl Default for TtmLanding {
    fn default() -> Self { Self::new() }
}
