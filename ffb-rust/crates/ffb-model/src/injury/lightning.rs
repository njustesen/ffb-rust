/// 1:1 translation of com.fumbbl.ffb.injury.Lightning.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct Lightning {
    base: InjuryType,
}

impl Lightning {
    pub fn new() -> Self {
        Self { base: InjuryType::new("lightning", false, SendToBoxReason::LIGHTNING) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }
}

impl Default for Lightning {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(Lightning::new().base().name(), "lightning");
    }

    #[test]
    fn worth_spps_is_false() {
        assert!(!Lightning::new().base().is_worth_spps());
    }

    #[test]
    fn send_to_box_reason_is_lightning() {
        use crate::model::send_to_box_reason::SendToBoxReason;
        assert_eq!(Lightning::new().base().send_to_box_reason(), SendToBoxReason::LIGHTNING);
    }
}
