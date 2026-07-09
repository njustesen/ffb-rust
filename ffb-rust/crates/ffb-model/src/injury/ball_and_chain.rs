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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(BallAndChain::new().base().name(), "ballAndChain");
    }

    #[test]
    fn worth_spps_is_false() {
        assert!(!BallAndChain::new().base().is_worth_spps());
    }

    #[test]
    fn is_block_is_true() {
        assert!(BallAndChain::new().is_block());
    }

    #[test]
    fn is_caused_by_opponent_is_false() {
        assert!(!BallAndChain::new().base().is_caused_by_opponent());
    }
}
