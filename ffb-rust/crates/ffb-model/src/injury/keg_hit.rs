/// 1:1 translation of com.fumbbl.ffb.injury.KegHit.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct KegHit {
    base: InjuryType,
}

impl KegHit {
    pub fn new() -> Self {
        Self { base: InjuryType::new("kegHit", false, SendToBoxReason::THROWN_KEG) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }
}

impl Default for KegHit {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(KegHit::new().base().name(), "kegHit");
    }

    #[test]
    fn worth_spps_is_false() {
        assert!(!KegHit::new().base().is_worth_spps());
    }

    #[test]
    fn send_to_box_reason_is_thrown_keg() {
        use crate::model::send_to_box_reason::SendToBoxReason;
        assert_eq!(KegHit::new().base().send_to_box_reason(), SendToBoxReason::THROWN_KEG);
    }
}
