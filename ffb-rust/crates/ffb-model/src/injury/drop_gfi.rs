/// 1:1 translation of com.fumbbl.ffb.injury.DropGfi.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct DropGfi {
    base: InjuryType,
}

impl DropGfi {
    pub fn new() -> Self {
        Self { base: InjuryType::new("dropGfi", false, SendToBoxReason::GFI_FAIL) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }
}

impl Default for DropGfi {
    fn default() -> Self { Self::new() }
}
