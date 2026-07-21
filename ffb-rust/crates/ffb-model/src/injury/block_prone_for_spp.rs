/// 1:1 translation of com.fumbbl.ffb.injury.BlockProneForSpp.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct BlockProneForSpp {
    base: InjuryType,
}

impl BlockProneForSpp {
    pub fn new() -> Self {
        Self { base: InjuryType::new("blockProneForSpp", true, SendToBoxReason::BLOCKED) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn is_caused_by_opponent(&self) -> bool { true }

    pub fn is_block(&self) -> bool { true }
}

impl Default for BlockProneForSpp {
    fn default() -> Self { Self::new() }
}
