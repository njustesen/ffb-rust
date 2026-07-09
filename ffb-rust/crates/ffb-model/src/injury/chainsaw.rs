/// 1:1 translation of com.fumbbl.ffb.injury.Chainsaw.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct Chainsaw {
    base: InjuryType,
}

impl Chainsaw {
    pub fn new() -> Self {
        Self { base: InjuryType::new("chainsaw", false, SendToBoxReason::CHAINSAW) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn is_caused_by_opponent(&self) -> bool { true }

    pub fn is_chainsaw(&self) -> bool { true }
}

impl Default for Chainsaw {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(Chainsaw::new().base().name(), "chainsaw");
    }

    #[test]
    fn worth_spps_is_false() {
        assert!(!Chainsaw::new().base().is_worth_spps());
    }

    #[test]
    fn is_caused_by_opponent_is_true() {
        assert!(Chainsaw::new().is_caused_by_opponent());
    }

    #[test]
    fn is_chainsaw_is_true() {
        assert!(Chainsaw::new().is_chainsaw());
    }
}
