/// 1:1 translation of com.fumbbl.ffb.injury.Stab.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct Stab {
    base: InjuryType,
}

impl Stab {
    pub fn new() -> Self {
        Self { base: InjuryType::new("stab", false, SendToBoxReason::STABBED) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn is_caused_by_opponent(&self) -> bool { true }

    pub fn is_stab(&self) -> bool { true }
}

impl Default for Stab {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(Stab::new().base().name(), "stab");
    }

    #[test]
    fn worth_spps_is_false() {
        assert!(!Stab::new().base().is_worth_spps());
    }

    #[test]
    fn is_caused_by_opponent_is_true() {
        assert!(Stab::new().is_caused_by_opponent());
    }

    #[test]
    fn is_stab_is_true() {
        assert!(Stab::new().is_stab());
    }
}
