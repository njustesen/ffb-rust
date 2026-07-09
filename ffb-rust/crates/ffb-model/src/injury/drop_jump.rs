/// 1:1 translation of com.fumbbl.ffb.injury.DropJump.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct DropJump {
    base: InjuryType,
}

impl DropJump {
    pub fn new() -> Self {
        Self { base: InjuryType::new("dropLeap", false, SendToBoxReason::JUMP_FAIL) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }
}

impl Default for DropJump {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(DropJump::new().base().name(), "dropLeap");
    }

    #[test]
    fn worth_spps_is_false() {
        assert!(!DropJump::new().base().is_worth_spps());
    }

    #[test]
    fn send_to_box_reason_is_jump_fail() {
        use crate::model::send_to_box_reason::SendToBoxReason;
        assert_eq!(DropJump::new().base().send_to_box_reason(), SendToBoxReason::JUMP_FAIL);
    }
}
