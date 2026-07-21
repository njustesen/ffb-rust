/// 1:1 translation of com.fumbbl.ffb.injury.KtmFumbleInjury.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct KtmFumbleInjury {
    base: InjuryType,
}

impl KtmFumbleInjury {
    pub fn new() -> Self {
        Self { base: InjuryType::new("ktmFumbleInjury", false, SendToBoxReason::KICKED) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn can_apo_ko_into_stun(&self) -> bool { false }
}

impl Default for KtmFumbleInjury {
    fn default() -> Self { Self::new() }
}
