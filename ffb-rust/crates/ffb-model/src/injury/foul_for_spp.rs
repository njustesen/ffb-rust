/// 1:1 translation of com.fumbbl.ffb.injury.FoulForSpp.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct FoulForSpp {
    base: InjuryType,
}

impl FoulForSpp {
    pub fn new() -> Self {
        Self { base: InjuryType::new("foulForSpp", true, SendToBoxReason::FOULED) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn should_play_fall_sound(&self) -> bool { false }

    pub fn is_foul(&self) -> bool { true }

    pub fn is_caused_by_opponent(&self) -> bool { true }
}

impl Default for FoulForSpp {
    fn default() -> Self { Self::new() }
}
