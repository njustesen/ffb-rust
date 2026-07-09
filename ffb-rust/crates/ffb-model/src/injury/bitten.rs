/// 1:1 translation of com.fumbbl.ffb.injury.Bitten.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct Bitten {
    base: InjuryType,
}

impl Bitten {
    pub fn new() -> Self {
        Self { base: InjuryType::new("bitten", false, SendToBoxReason::BITTEN) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }
}

impl Default for Bitten {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(Bitten::new().base().name(), "bitten");
    }

    #[test]
    fn worth_spps_is_false() {
        assert!(!Bitten::new().base().is_worth_spps());
    }

    #[test]
    fn send_to_box_reason_is_bitten() {
        use crate::model::send_to_box_reason::SendToBoxReason;
        assert_eq!(Bitten::new().base().send_to_box_reason(), SendToBoxReason::BITTEN);
    }
}
