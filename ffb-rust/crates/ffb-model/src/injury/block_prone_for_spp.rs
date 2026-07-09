/// 1:1 translation of com.fumbbl.ffb.injury.BlockProneForSpp.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct BlockProneForSpp {
    base: InjuryType,
}

impl BlockProneForSpp {
    pub fn new() -> Self {
        Self { base: InjuryType::new("blockProneForSpp", true, SendToBoxReason::BLOCKED) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn is_caused_by_opponent(&self) -> bool { true }

    pub fn is_block(&self) -> bool { true }
}

impl Default for BlockProneForSpp {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(BlockProneForSpp::new().base().name(), "blockProneForSpp");
    }

    #[test]
    fn worth_spps_is_true() {
        assert!(BlockProneForSpp::new().base().is_worth_spps());
    }

    #[test]
    fn is_caused_by_opponent_is_true() {
        assert!(BlockProneForSpp::new().is_caused_by_opponent());
    }

    #[test]
    fn is_block_is_true() {
        assert!(BlockProneForSpp::new().is_block());
    }
}
