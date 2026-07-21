/// 1:1 translation of com.fumbbl.ffb.injury.Fireball.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct Fireball {
    base: InjuryType,
}

impl Fireball {
    pub fn new() -> Self {
        Self { base: InjuryType::new("fireball", false, SendToBoxReason::FIREBALL) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }
}

impl Default for Fireball {
    fn default() -> Self { Self::new() }
}
