/// 1:1 translation of com.fumbbl.ffb.injury.Stab.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct Stab {
    base: InjuryType,
}

impl Stab {
    pub fn new() -> Self {
        Self { base: InjuryType::new("stab", false, SendToBoxReason::STABBED) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn is_caused_by_opponent(&self) -> bool { true }

    pub fn is_stab(&self) -> bool { true }
}

impl Default for Stab {
    fn default() -> Self { Self::new() }
}
