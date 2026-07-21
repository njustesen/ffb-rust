/// 1:1 translation of com.fumbbl.ffb.injury.EatPlayer.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct EatPlayer {
    base: InjuryType,
}

impl EatPlayer {
    pub fn new() -> Self {
        Self { base: InjuryType::new("eatPlayer", false, SendToBoxReason::EATEN) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn can_use_apo(&self) -> bool { false }
}

impl Default for EatPlayer {
    fn default() -> Self { Self::new() }
}
