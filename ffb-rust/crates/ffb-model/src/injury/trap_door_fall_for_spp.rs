/// 1:1 translation of com.fumbbl.ffb.injury.TrapDoorFallForSpp.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct TrapDoorFallForSpp {
    base: InjuryType,
}

impl TrapDoorFallForSpp {
    pub fn new() -> Self {
        Self { base: InjuryType::new("trapdoorFallForSpp", true, SendToBoxReason::TRAP_DOOR_FALL) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn can_apo_ko_into_stun(&self) -> bool { false }

    pub fn falling_down_causes_turnover(&self) -> bool { false }

    pub fn is_caused_by_opponent(&self) -> bool { true }
}

impl Default for TrapDoorFallForSpp {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(TrapDoorFallForSpp::new().base().name(), "trapdoorFallForSpp");
    }

    #[test]
    fn worth_spps_is_true() {
        assert!(TrapDoorFallForSpp::new().base().is_worth_spps());
    }

    #[test]
    fn can_apo_ko_into_stun_is_false() {
        assert!(!TrapDoorFallForSpp::new().can_apo_ko_into_stun());
    }

    #[test]
    fn falling_down_causes_turnover_is_false() {
        assert!(!TrapDoorFallForSpp::new().falling_down_causes_turnover());
    }

    #[test]
    fn is_caused_by_opponent_is_true() {
        assert!(TrapDoorFallForSpp::new().is_caused_by_opponent());
    }
}
