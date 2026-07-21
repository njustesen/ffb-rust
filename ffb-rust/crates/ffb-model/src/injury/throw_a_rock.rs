/// 1:1 translation of com.fumbbl.ffb.injury.ThrowARock.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct ThrowARock {
    base: InjuryType,
}

impl ThrowARock {
    pub fn new() -> Self {
        Self { base: InjuryType::new("throwARock", false, SendToBoxReason::HIT_BY_ROCK) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }
}

impl Default for ThrowARock {
    fn default() -> Self { Self::new() }
}
