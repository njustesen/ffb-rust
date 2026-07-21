/// 1:1 translation of com.fumbbl.ffb.injury.BreatheFire.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct BreatheFire {
    base: InjuryType,
}

impl BreatheFire {
    pub fn new() -> Self {
        Self { base: InjuryType::new("breatheFire", false, SendToBoxReason::BREATHE_FIRE) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn is_vomit_like(&self) -> bool { true }

    pub fn is_caused_by_opponent(&self) -> bool { true }
}

impl Default for BreatheFire {
    fn default() -> Self { Self::new() }
}
