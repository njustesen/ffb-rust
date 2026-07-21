/// 1:1 translation of com.fumbbl.ffb.injury.Bomb.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct Bomb {
    base: InjuryType,
}

impl Bomb {
    pub fn new() -> Self {
        Self { base: InjuryType::new("bomb", false, SendToBoxReason::BOMB) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }
}

impl Default for Bomb {
    fn default() -> Self { Self::new() }
}
