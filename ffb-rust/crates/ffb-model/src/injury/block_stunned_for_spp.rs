/// 1:1 translation of com.fumbbl.ffb.injury.BlockStunnedForSpp.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct BlockStunnedForSpp {
    base: InjuryType,
}

impl BlockStunnedForSpp {
    pub fn new() -> Self {
        Self { base: InjuryType::new("blockStunnedForSpp", true, SendToBoxReason::BLOCKED) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn is_caused_by_opponent(&self) -> bool { true }

    pub fn is_block(&self) -> bool { true }
}

impl Default for BlockStunnedForSpp {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(BlockStunnedForSpp::new().base().name(), "blockStunnedForSpp");
    }

    #[test]
    fn worth_spps_is_true() {
        assert!(BlockStunnedForSpp::new().base().is_worth_spps());
    }

    #[test]
    fn is_caused_by_opponent_is_true() {
        assert!(BlockStunnedForSpp::new().is_caused_by_opponent());
    }

    #[test]
    fn is_block_is_true() {
        assert!(BlockStunnedForSpp::new().is_block());
    }
}
