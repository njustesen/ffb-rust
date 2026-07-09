/// 1:1 translation of com.fumbbl.ffb.injury.BlockProne.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct BlockProne {
    base: InjuryType,
}

impl BlockProne {
    pub fn new() -> Self {
        Self { base: InjuryType::new("blockProne", false, SendToBoxReason::BLOCKED) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn is_caused_by_opponent(&self) -> bool { true }

    pub fn is_block(&self) -> bool { true }
}

impl Default for BlockProne {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(BlockProne::new().base().name(), "blockProne");
    }

    #[test]
    fn worth_spps_is_false() {
        assert!(!BlockProne::new().base().is_worth_spps());
    }

    #[test]
    fn is_caused_by_opponent_is_true() {
        assert!(BlockProne::new().is_caused_by_opponent());
    }

    #[test]
    fn is_block_is_true() {
        assert!(BlockProne::new().is_block());
    }
}
