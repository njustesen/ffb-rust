/// 1:1 translation of com.fumbbl.ffb.injury.BlockStunned.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct BlockStunned {
    base: InjuryType,
}

impl BlockStunned {
    pub fn new() -> Self {
        Self { base: InjuryType::new("blockStunned", false, SendToBoxReason::BLOCKED) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn is_caused_by_opponent(&self) -> bool { true }

    pub fn is_block(&self) -> bool { true }
}

impl Default for BlockStunned {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(BlockStunned::new().base().name(), "blockStunned");
    }

    #[test]
    fn worth_spps_is_false() {
        assert!(!BlockStunned::new().base().is_worth_spps());
    }

    #[test]
    fn is_caused_by_opponent_is_true() {
        assert!(BlockStunned::new().is_caused_by_opponent());
    }

    #[test]
    fn is_block_is_true() {
        assert!(BlockStunned::new().is_block());
    }
}
