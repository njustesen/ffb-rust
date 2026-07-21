/// 1:1 translation of com.fumbbl.ffb.injury.DropDodge.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct DropDodge {
    base: InjuryType,
}

impl DropDodge {
    pub fn new() -> Self {
        Self { base: InjuryType::new("dropDodge", false, SendToBoxReason::DODGE_FAIL) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }
}

impl Default for DropDodge {
    fn default() -> Self { Self::new() }
}
