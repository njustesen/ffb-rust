/// 1:1 translation of com.fumbbl.ffb.injury.Bomb.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct Bomb {
    base: InjuryType,
}

impl Bomb {
    pub fn new() -> Self {
        Self { base: InjuryType::new("bomb", false, SendToBoxReason::BOMB) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }
}

impl Default for Bomb {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(Bomb::new().base().name(), "bomb");
    }

    #[test]
    fn worth_spps_is_false() {
        assert!(!Bomb::new().base().is_worth_spps());
    }

    #[test]
    fn send_to_box_reason_is_bomb() {
        use crate::model::send_to_box_reason::SendToBoxReason;
        assert_eq!(Bomb::new().base().send_to_box_reason(), SendToBoxReason::BOMB);
    }
}
