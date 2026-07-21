/// 1:1 translation of com.fumbbl.ffb.injury.PilingOnArmour.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct PilingOnArmour {
    base: InjuryType,
}

impl PilingOnArmour {
    pub fn new() -> Self {
        Self { base: InjuryType::new("pilingOnArmor", true, SendToBoxReason::PILED_ON) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn is_caused_by_opponent(&self) -> bool { true }
}

impl Default for PilingOnArmour {
    fn default() -> Self { Self::new() }
}
