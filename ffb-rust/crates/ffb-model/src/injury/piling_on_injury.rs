/// 1:1 translation of com.fumbbl.ffb.injury.PilingOnInjury.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct PilingOnInjury {
    base: InjuryType,
}

impl PilingOnInjury {
    pub fn new() -> Self {
        Self { base: InjuryType::new("pilingOnInjury", true, SendToBoxReason::PILED_ON) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn is_caused_by_opponent(&self) -> bool { true }
}

impl Default for PilingOnInjury {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(PilingOnInjury::new().base().name(), "pilingOnInjury");
    }

    #[test]
    fn worth_spps_is_true() {
        assert!(PilingOnInjury::new().base().is_worth_spps());
    }

    #[test]
    fn is_caused_by_opponent_is_true() {
        assert!(PilingOnInjury::new().is_caused_by_opponent());
    }
}
