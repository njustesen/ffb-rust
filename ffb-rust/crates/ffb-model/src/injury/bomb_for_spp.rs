/// 1:1 translation of com.fumbbl.ffb.injury.BombForSpp.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct BombForSpp {
    base: InjuryType,
}

impl BombForSpp {
    pub fn new() -> Self {
        Self { base: InjuryType::new("bombForSpp", true, SendToBoxReason::BOMB) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn is_caused_by_opponent(&self) -> bool { true }
}

impl Default for BombForSpp {
    fn default() -> Self { Self::new() }
}
