/// 1:1 translation of com.fumbbl.ffb.injury.Fireball.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct Fireball {
    base: InjuryType,
}

impl Fireball {
    pub fn new() -> Self {
        Self { base: InjuryType::new("fireball", false, SendToBoxReason::FIREBALL) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }
}

impl Default for Fireball {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(Fireball::new().base().name(), "fireball");
    }

    #[test]
    fn worth_spps_is_false() {
        assert!(!Fireball::new().base().is_worth_spps());
    }

    #[test]
    fn send_to_box_reason_is_fireball() {
        use crate::model::send_to_box_reason::SendToBoxReason;
        assert_eq!(Fireball::new().base().send_to_box_reason(), SendToBoxReason::FIREBALL);
    }
}
