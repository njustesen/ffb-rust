/// 1:1 translation of com.fumbbl.ffb.injury.PilingOnInjury.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct PilingOnInjury {
    base: InjuryType,
}

impl PilingOnInjury {
    pub fn new() -> Self {
        Self { base: InjuryType::new("pilingOnInjury", true, SendToBoxReason::PILED_ON) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn is_caused_by_opponent(&self) -> bool { true }
}

impl Default for PilingOnInjury {
    fn default() -> Self { Self::new() }
}
