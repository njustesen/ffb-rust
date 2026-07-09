/// 1:1 translation of com.fumbbl.ffb.injury.KtmInjury.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct KtmInjury {
    base: InjuryType,
}

impl KtmInjury {
    pub fn new() -> Self {
        Self { base: InjuryType::new("ktmInjury", false, SendToBoxReason::KICKED) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }
}

impl Default for KtmInjury {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(KtmInjury::new().base().name(), "ktmInjury");
    }

    #[test]
    fn worth_spps_is_false() {
        assert!(!KtmInjury::new().base().is_worth_spps());
    }

    #[test]
    fn send_to_box_reason_is_kicked() {
        use crate::model::send_to_box_reason::SendToBoxReason;
        assert_eq!(KtmInjury::new().base().send_to_box_reason(), SendToBoxReason::KICKED);
    }
}
