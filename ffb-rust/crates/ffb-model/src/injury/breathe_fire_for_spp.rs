/// 1:1 translation of com.fumbbl.ffb.injury.BreatheFireForSpp.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct BreatheFireForSpp {
    base: InjuryType,
}

impl BreatheFireForSpp {
    pub fn new() -> Self {
        Self { base: InjuryType::new("breatheFireForSpp", true, SendToBoxReason::BREATHE_FIRE) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn is_vomit_like(&self) -> bool { true }

    pub fn is_caused_by_opponent(&self) -> bool { true }
}

impl Default for BreatheFireForSpp {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(BreatheFireForSpp::new().base().name(), "breatheFireForSpp");
    }

    #[test]
    fn worth_spps_is_true() {
        assert!(BreatheFireForSpp::new().base().is_worth_spps());
    }

    #[test]
    fn is_vomit_like_is_true() {
        assert!(BreatheFireForSpp::new().is_vomit_like());
    }

    #[test]
    fn is_caused_by_opponent_is_true() {
        assert!(BreatheFireForSpp::new().is_caused_by_opponent());
    }
}
