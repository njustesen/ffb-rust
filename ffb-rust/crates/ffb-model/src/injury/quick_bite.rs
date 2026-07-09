/// 1:1 translation of com.fumbbl.ffb.injury.QuickBite.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct QuickBite {
    base: InjuryType,
}

impl QuickBite {
    pub fn new() -> Self {
        Self { base: InjuryType::new("quickBite", false, SendToBoxReason::QUICK_BITE) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn is_caused_by_opponent(&self) -> bool { true }
}

impl Default for QuickBite {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(QuickBite::new().base().name(), "quickBite");
    }

    #[test]
    fn worth_spps_is_false() {
        assert!(!QuickBite::new().base().is_worth_spps());
    }

    #[test]
    fn is_caused_by_opponent_is_true() {
        assert!(QuickBite::new().is_caused_by_opponent());
    }
}
