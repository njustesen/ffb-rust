/// 1:1 translation of com.fumbbl.ffb.injury.BlockStunned.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct BlockStunned {
    base: InjuryType,
}

impl BlockStunned {
    pub fn new() -> Self {
        Self { base: InjuryType::new("blockStunned", false, SendToBoxReason::BLOCKED) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn is_caused_by_opponent(&self) -> bool { true }

    pub fn is_block(&self) -> bool { true }
}

impl Default for BlockStunned {
    fn default() -> Self { Self::new() }
}
