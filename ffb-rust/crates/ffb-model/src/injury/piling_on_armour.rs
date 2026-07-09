/// 1:1 translation of com.fumbbl.ffb.injury.PilingOnArmour.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct PilingOnArmour {
    base: InjuryType,
}

impl PilingOnArmour {
    pub fn new() -> Self {
        Self { base: InjuryType::new("pilingOnArmor", true, SendToBoxReason::PILED_ON) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn is_caused_by_opponent(&self) -> bool { true }
}

impl Default for PilingOnArmour {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(PilingOnArmour::new().base().name(), "pilingOnArmor");
    }

    #[test]
    fn worth_spps_is_true() {
        assert!(PilingOnArmour::new().base().is_worth_spps());
    }

    #[test]
    fn is_caused_by_opponent_is_true() {
        assert!(PilingOnArmour::new().is_caused_by_opponent());
    }
}
