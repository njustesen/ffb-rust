/// 1:1 translation of com.fumbbl.ffb.injury.Block.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct Block {
    base: InjuryType,
}

impl Block {
    pub fn new() -> Self {
        Self { base: InjuryType::new("block", true, SendToBoxReason::BLOCKED) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn is_caused_by_opponent(&self) -> bool { true }

    pub fn is_block(&self) -> bool { true }
}

impl Default for Block {
    fn default() -> Self { Self::new() }
}
