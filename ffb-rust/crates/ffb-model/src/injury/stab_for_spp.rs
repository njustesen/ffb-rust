/// 1:1 translation of com.fumbbl.ffb.injury.StabForSpp.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct StabForSpp {
    base: InjuryType,
}

impl StabForSpp {
    pub fn new() -> Self {
        Self { base: InjuryType::new("stabForSpp", true, SendToBoxReason::STABBED) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn is_caused_by_opponent(&self) -> bool { true }

    pub fn is_stab(&self) -> bool { true }
}

impl Default for StabForSpp {
    fn default() -> Self { Self::new() }
}
