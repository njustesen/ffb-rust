/// 1:1 translation of com.fumbbl.ffb.injury.DropDodgeForSpp.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct DropDodgeForSpp {
    base: InjuryType,
}

impl DropDodgeForSpp {
    pub fn new() -> Self {
        Self { base: InjuryType::new("dropDodgeForSpp", true, SendToBoxReason::DODGE_FAIL) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn is_caused_by_opponent(&self) -> bool { true }
}

impl Default for DropDodgeForSpp {
    fn default() -> Self { Self::new() }
}
