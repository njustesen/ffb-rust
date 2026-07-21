/// 1:1 translation of com.fumbbl.ffb.injury.TtmHitPlayer.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct TtmHitPlayer {
    base: InjuryType,
}

impl TtmHitPlayer {
    pub fn new() -> Self {
        Self { base: InjuryType::new("ttmHitPlayer", false, SendToBoxReason::HIT_BY_THROWN_PLAYER) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }
}

impl Default for TtmHitPlayer {
    fn default() -> Self { Self::new() }
}
