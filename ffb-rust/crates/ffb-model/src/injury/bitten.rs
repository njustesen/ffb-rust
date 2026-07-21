/// 1:1 translation of com.fumbbl.ffb.injury.Bitten.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct Bitten {
    base: InjuryType,
}

impl Bitten {
    pub fn new() -> Self {
        Self { base: InjuryType::new("bitten", false, SendToBoxReason::BITTEN) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }
}

impl Default for Bitten {
    fn default() -> Self { Self::new() }
}
