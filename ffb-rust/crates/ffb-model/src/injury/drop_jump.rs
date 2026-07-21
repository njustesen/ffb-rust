/// 1:1 translation of com.fumbbl.ffb.injury.DropJump.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct DropJump {
    base: InjuryType,
}

impl DropJump {
    pub fn new() -> Self {
        Self { base: InjuryType::new("dropLeap", false, SendToBoxReason::JUMP_FAIL) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }
}

impl Default for DropJump {
    fn default() -> Self { Self::new() }
}
