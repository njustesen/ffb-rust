/// 1:1 translation of com.fumbbl.ffb.injury.BombForSpp.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct BombForSpp {
    base: InjuryType,
}

impl BombForSpp {
    pub fn new() -> Self {
        Self { base: InjuryType::new("bombForSpp", true, SendToBoxReason::BOMB) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn is_caused_by_opponent(&self) -> bool { true }
}

impl Default for BombForSpp {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(BombForSpp::new().base().name(), "bombForSpp");
    }

    #[test]
    fn worth_spps_is_true() {
        assert!(BombForSpp::new().base().is_worth_spps());
    }

    #[test]
    fn is_caused_by_opponent_is_true() {
        assert!(BombForSpp::new().is_caused_by_opponent());
    }
}
