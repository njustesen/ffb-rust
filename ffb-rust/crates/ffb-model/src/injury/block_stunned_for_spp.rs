/// 1:1 translation of com.fumbbl.ffb.injury.BlockStunnedForSpp.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct BlockStunnedForSpp {
    base: InjuryType,
}

impl BlockStunnedForSpp {
    pub fn new() -> Self {
        Self { base: InjuryType::new("blockStunnedForSpp", true, SendToBoxReason::BLOCKED) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn is_caused_by_opponent(&self) -> bool { true }

    pub fn is_block(&self) -> bool { true }
}

impl Default for BlockStunnedForSpp {
    fn default() -> Self { Self::new() }
}
