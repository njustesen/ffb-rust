/// 1:1 translation of com.fumbbl.ffb.injury.Foul.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct Foul {
    base: InjuryType,
}

impl Foul {
    pub fn new() -> Self {
        Self { base: InjuryType::new("foul", false, SendToBoxReason::FOULED) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn should_play_fall_sound(&self) -> bool { false }

    pub fn is_foul(&self) -> bool { true }
}

impl Default for Foul {
    fn default() -> Self { Self::new() }
}
