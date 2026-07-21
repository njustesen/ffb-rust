/// 1:1 translation of com.fumbbl.ffb.injury.TrapDoorFall.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct TrapDoorFall {
    base: InjuryType,
}

impl TrapDoorFall {
    pub fn new() -> Self {
        Self { base: InjuryType::new("trapdoorFall", false, SendToBoxReason::TRAP_DOOR_FALL) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn can_apo_ko_into_stun(&self) -> bool { false }

    pub fn falling_down_causes_turnover(&self) -> bool { false }
}

impl Default for TrapDoorFall {
    fn default() -> Self { Self::new() }
}
