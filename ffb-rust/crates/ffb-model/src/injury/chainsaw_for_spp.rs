/// 1:1 translation of com.fumbbl.ffb.injury.ChainsawForSpp.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct ChainsawForSpp {
    base: InjuryType,
}

impl ChainsawForSpp {
    pub fn new() -> Self {
        Self { base: InjuryType::new("chainsawForSpp", true, SendToBoxReason::CHAINSAW) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn is_caused_by_opponent(&self) -> bool { true }

    pub fn is_chainsaw(&self) -> bool { true }
}

impl Default for ChainsawForSpp {
    fn default() -> Self { Self::new() }
}
