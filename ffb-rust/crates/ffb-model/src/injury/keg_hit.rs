/// 1:1 translation of com.fumbbl.ffb.injury.KegHit.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct KegHit {
    base: InjuryType,
}

impl KegHit {
    pub fn new() -> Self {
        Self { base: InjuryType::new("kegHit", false, SendToBoxReason::THROWN_KEG) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }
}

impl Default for KegHit {
    fn default() -> Self { Self::new() }
}
