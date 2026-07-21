/// 1:1 translation of com.fumbbl.ffb.injury.ThenIStartedBlastin.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct ThenIStartedBlastin {
    base: InjuryType,
}

impl ThenIStartedBlastin {
    pub fn new() -> Self {
        Self { base: InjuryType::new("startedBlastin", false, SendToBoxReason::THEN_I_STARTED_BLASTIN) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn is_caused_by_opponent(&self) -> bool { true }
}

impl Default for ThenIStartedBlastin {
    fn default() -> Self { Self::new() }
}
