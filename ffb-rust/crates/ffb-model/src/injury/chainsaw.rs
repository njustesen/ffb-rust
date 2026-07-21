/// 1:1 translation of com.fumbbl.ffb.injury.Chainsaw.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct Chainsaw {
    base: InjuryType,
}

impl Chainsaw {
    pub fn new() -> Self {
        Self { base: InjuryType::new("chainsaw", false, SendToBoxReason::CHAINSAW) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn is_caused_by_opponent(&self) -> bool { true }

    pub fn is_chainsaw(&self) -> bool { true }
}

impl Default for Chainsaw {
    fn default() -> Self { Self::new() }
}
