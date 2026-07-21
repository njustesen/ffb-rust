/// 1:1 translation of com.fumbbl.ffb.injury.BreatheFireForSpp.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct BreatheFireForSpp {
    base: InjuryType,
}

impl BreatheFireForSpp {
    pub fn new() -> Self {
        Self { base: InjuryType::new("breatheFireForSpp", true, SendToBoxReason::BREATHE_FIRE) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn is_vomit_like(&self) -> bool { true }

    pub fn is_caused_by_opponent(&self) -> bool { true }
}

impl Default for BreatheFireForSpp {
    fn default() -> Self { Self::new() }
}
