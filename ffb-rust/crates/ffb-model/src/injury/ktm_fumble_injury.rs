/// 1:1 translation of com.fumbbl.ffb.injury.KtmFumbleInjury.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct KtmFumbleInjury {
    base: InjuryType,
}

impl KtmFumbleInjury {
    pub fn new() -> Self {
        Self { base: InjuryType::new("ktmFumbleInjury", false, SendToBoxReason::KICKED) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    pub fn can_apo_ko_into_stun(&self) -> bool { false }
}

impl Default for KtmFumbleInjury {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(KtmFumbleInjury::new().base().name(), "ktmFumbleInjury");
    }

    #[test]
    fn worth_spps_is_false() {
        assert!(!KtmFumbleInjury::new().base().is_worth_spps());
    }

    #[test]
    fn can_apo_ko_into_stun_is_false() {
        assert!(!KtmFumbleInjury::new().can_apo_ko_into_stun());
    }
}
