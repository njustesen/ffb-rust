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
