/// 1:1 translation of com.fumbbl.ffb.injury.PilingOnKnockedOut.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct PilingOnKnockedOut {
    base: InjuryType,
}

impl PilingOnKnockedOut {
    pub fn new() -> Self {
        Self { base: InjuryType::new("pilingOnKnockedOut", false, SendToBoxReason::KO_ON_PILING_ON) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn can_use_apo(&self) -> bool { false }

    pub fn is_caused_by_opponent(&self) -> bool { true }
}

impl Default for PilingOnKnockedOut {
    fn default() -> Self { Self::new() }
}
