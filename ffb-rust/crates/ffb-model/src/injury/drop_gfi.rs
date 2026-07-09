/// 1:1 translation of com.fumbbl.ffb.injury.DropGfi.
use crate::injury::injury_type::InjuryType;
use crate::model::send_to_box_reason::SendToBoxReason;

pub struct DropGfi {
    base: InjuryType,
}

impl DropGfi {
    pub fn new() -> Self {
        Self { base: InjuryType::new("dropGfi", false, SendToBoxReason::GFI_FAIL) }
    }

    pub fn base(&self) -> &InjuryType { &self.base }
}

impl Default for DropGfi {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(DropGfi::new().base().name(), "dropGfi");
    }

    #[test]
    fn worth_spps_is_false() {
        assert!(!DropGfi::new().base().is_worth_spps());
    }

    #[test]
    fn send_to_box_reason_is_gfi_fail() {
        use crate::model::send_to_box_reason::SendToBoxReason;
        assert_eq!(DropGfi::new().base().send_to_box_reason(), SendToBoxReason::GFI_FAIL);
    }
}
