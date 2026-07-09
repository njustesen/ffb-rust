/// 1:1 translation of com.fumbbl.ffb.injury.Saboteur.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct Saboteur {
    base: InjuryType,
}

impl Saboteur {
    pub fn new() -> Self {
        Self { base: InjuryType::new("saboteur", false, SendToBoxReason::SABOTEUR) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn is_caused_by_opponent(&self) -> bool { false }

    pub fn can_use_apo(&self) -> bool { false }

    pub fn falling_down_causes_turnover(&self) -> bool { false }
}

impl Default for Saboteur {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(Saboteur::new().base().name(), "saboteur");
    }

    #[test]
    fn worth_spps_is_false() {
        assert!(!Saboteur::new().base().is_worth_spps());
    }

    #[test]
    fn is_caused_by_opponent_is_false() {
        assert!(!Saboteur::new().is_caused_by_opponent());
    }

    #[test]
    fn can_use_apo_is_false() {
        assert!(!Saboteur::new().can_use_apo());
    }

    #[test]
    fn falling_down_causes_turnover_is_false() {
        assert!(!Saboteur::new().falling_down_causes_turnover());
    }
}
