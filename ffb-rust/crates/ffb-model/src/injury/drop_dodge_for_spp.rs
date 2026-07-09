/// 1:1 translation of com.fumbbl.ffb.injury.DropDodgeForSpp.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct DropDodgeForSpp {
    base: InjuryType,
}

impl DropDodgeForSpp {
    pub fn new() -> Self {
        Self { base: InjuryType::new("dropDodgeForSpp", true, SendToBoxReason::DODGE_FAIL) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn is_caused_by_opponent(&self) -> bool { true }
}

impl Default for DropDodgeForSpp {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(DropDodgeForSpp::new().base().name(), "dropDodgeForSpp");
    }

    #[test]
    fn worth_spps_is_true() {
        assert!(DropDodgeForSpp::new().base().is_worth_spps());
    }

    #[test]
    fn is_caused_by_opponent_is_true() {
        assert!(DropDodgeForSpp::new().is_caused_by_opponent());
    }
}
