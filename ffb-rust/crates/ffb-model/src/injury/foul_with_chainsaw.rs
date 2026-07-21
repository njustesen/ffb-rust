/// 1:1 translation of com.fumbbl.ffb.injury.FoulWithChainsaw.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct FoulWithChainsaw {
    base: InjuryType,
}

impl FoulWithChainsaw {
    pub fn new() -> Self {
        Self { base: InjuryType::new("foulWithChainsaw", false, SendToBoxReason::FOULED) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn should_play_fall_sound(&self) -> bool { false }

    pub fn is_foul(&self) -> bool { true }

    pub fn is_chainsaw(&self) -> bool { true }
}

impl Default for FoulWithChainsaw {
    fn default() -> Self { Self::new() }
}
