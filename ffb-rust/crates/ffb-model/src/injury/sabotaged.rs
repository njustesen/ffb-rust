/// 1:1 translation of com.fumbbl.ffb.injury.Sabotaged.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct Sabotaged {
    base: InjuryType,
}

impl Sabotaged {
    pub fn new() -> Self {
        Self { base: InjuryType::new("sabotaged", false, SendToBoxReason::SABOTAGED) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    /// Java explicitly returns false — same as base, included for clarity.
    pub fn is_caused_by_opponent(&self) -> bool { false }
}

impl Default for Sabotaged {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(Sabotaged::new().base().name(), "sabotaged");
    }

    #[test]
    fn worth_spps_is_false() {
        assert!(!Sabotaged::new().base().is_worth_spps());
    }

    #[test]
    fn is_caused_by_opponent_is_false() {
        assert!(!Sabotaged::new().is_caused_by_opponent());
    }
}
