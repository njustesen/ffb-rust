/// 1:1 translation of com.fumbbl.ffb.injury.FoulForSppWithChainsaw.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct FoulForSppWithChainsaw {
    base: InjuryType,
}

impl FoulForSppWithChainsaw {
    pub fn new() -> Self {
        Self { base: InjuryType::new("foulForSppWithChainsaw", true, SendToBoxReason::FOULED) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn should_play_fall_sound(&self) -> bool { false }

    pub fn is_foul(&self) -> bool { true }

    pub fn is_caused_by_opponent(&self) -> bool { true }

    pub fn is_chainsaw(&self) -> bool { true }
}

impl Default for FoulForSppWithChainsaw {
    fn default() -> Self { Self::new() }
}
