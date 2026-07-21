/// 1:1 translation of com.fumbbl.ffb.injury.Lightning.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct Lightning {
    base: InjuryType,
}

impl Lightning {
    pub fn new() -> Self {
        Self { base: InjuryType::new("lightning", false, SendToBoxReason::LIGHTNING) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }
}

impl Default for Lightning {
    fn default() -> Self { Self::new() }
}
