/// 1:1 translation of com.fumbbl.ffb.injury.DropDodge.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct DropDodge {
    base: InjuryType,
}

impl DropDodge {
    pub fn new() -> Self {
        Self { base: InjuryType::new("dropDodge", false, SendToBoxReason::DODGE_FAIL) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }
}

impl Default for DropDodge {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(DropDodge::new().base().name(), "dropDodge");
    }

    #[test]
    fn worth_spps_is_false() {
        assert!(!DropDodge::new().base().is_worth_spps());
    }

    #[test]
    fn send_to_box_reason_is_dodge_fail() {
        use crate::model::send_to_box_reason::SendToBoxReason;
        assert_eq!(DropDodge::new().base().send_to_box_reason(), SendToBoxReason::DODGE_FAIL);
    }
}
