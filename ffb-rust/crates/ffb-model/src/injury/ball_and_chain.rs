/// 1:1 translation of com.fumbbl.ffb.injury.BallAndChain.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct BallAndChain {
    base: InjuryType,
}

impl BallAndChain {
    pub fn new() -> Self {
        Self { base: InjuryType::new("ballAndChain", false, SendToBoxReason::BALL_AND_CHAIN) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn is_block(&self) -> bool { true }
}

impl Default for BallAndChain {
    fn default() -> Self { Self::new() }
}
