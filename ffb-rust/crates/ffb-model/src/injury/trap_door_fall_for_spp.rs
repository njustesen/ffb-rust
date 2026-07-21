/// 1:1 translation of com.fumbbl.ffb.injury.TrapDoorFallForSpp.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct TrapDoorFallForSpp {
    base: InjuryType,
}

impl TrapDoorFallForSpp {
    pub fn new() -> Self {
        Self { base: InjuryType::new("trapdoorFallForSpp", true, SendToBoxReason::TRAP_DOOR_FALL) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn can_apo_ko_into_stun(&self) -> bool { false }

    pub fn falling_down_causes_turnover(&self) -> bool { false }

    pub fn is_caused_by_opponent(&self) -> bool { true }
}

impl Default for TrapDoorFallForSpp {
    fn default() -> Self { Self::new() }
}
