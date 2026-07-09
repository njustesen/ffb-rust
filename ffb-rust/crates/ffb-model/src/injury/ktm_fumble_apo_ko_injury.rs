/// 1:1 translation of com.fumbbl.ffb.injury.KtmFumbleApoKoInjury.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct KtmFumbleApoKoInjury {
    base: InjuryType,
}

impl KtmFumbleApoKoInjury {
    pub fn new() -> Self {
        Self { base: InjuryType::new("ktmFumbleApoKoInjury", false, SendToBoxReason::KICKED) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }

    /// Java explicitly returns true — same as base, included for clarity.
    pub fn can_apo_ko_into_stun(&self) -> bool { true }
}

impl Default for KtmFumbleApoKoInjury {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(KtmFumbleApoKoInjury::new().base().name(), "ktmFumbleApoKoInjury");
    }

    #[test]
    fn worth_spps_is_false() {
        assert!(!KtmFumbleApoKoInjury::new().base().is_worth_spps());
    }

    #[test]
    fn can_apo_ko_into_stun_is_true() {
        assert!(KtmFumbleApoKoInjury::new().can_apo_ko_into_stun());
    }
}
