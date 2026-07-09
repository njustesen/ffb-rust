/// 1:1 translation of com.fumbbl.ffb.injury.ThrowARock.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct ThrowARock {
    base: InjuryType,
}

impl ThrowARock {
    pub fn new() -> Self {
        Self { base: InjuryType::new("throwARock", false, SendToBoxReason::HIT_BY_ROCK) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }
}

impl Default for ThrowARock {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(ThrowARock::new().base().name(), "throwARock");
    }

    #[test]
    fn worth_spps_is_false() {
        assert!(!ThrowARock::new().base().is_worth_spps());
    }

    #[test]
    fn send_to_box_reason_is_hit_by_rock() {
        use crate::model::send_to_box_reason::SendToBoxReason;
        assert_eq!(ThrowARock::new().base().send_to_box_reason(), SendToBoxReason::HIT_BY_ROCK);
    }
}
