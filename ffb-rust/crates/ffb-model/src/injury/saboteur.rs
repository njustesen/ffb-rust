/// 1:1 translation of com.fumbbl.ffb.injury.Saboteur.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct Saboteur {
    base: InjuryType,
}

impl Saboteur {
    pub fn new() -> Self {
        Self { base: InjuryType::new("saboteur", false, SendToBoxReason::SABOTEUR) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn is_caused_by_opponent(&self) -> bool { false }

    pub fn can_use_apo(&self) -> bool { false }

    pub fn falling_down_causes_turnover(&self) -> bool { false }
}

impl Default for Saboteur {
    fn default() -> Self { Self::new() }
}
