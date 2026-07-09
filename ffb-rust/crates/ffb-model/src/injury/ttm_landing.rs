/// 1:1 translation of com.fumbbl.ffb.injury.TtmLanding.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct TtmLanding {
    base: InjuryType,
}

impl TtmLanding {
    pub fn new() -> Self {
        Self { base: InjuryType::new("ttmLanding", false, SendToBoxReason::LANDING_FAIL) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }
}

impl Default for TtmLanding {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(TtmLanding::new().base().name(), "ttmLanding");
    }

    #[test]
    fn worth_spps_is_false() {
        assert!(!TtmLanding::new().base().is_worth_spps());
    }

    #[test]
    fn send_to_box_reason_is_landing_fail() {
        use crate::model::send_to_box_reason::SendToBoxReason;
        assert_eq!(TtmLanding::new().base().send_to_box_reason(), SendToBoxReason::LANDING_FAIL);
    }
}
