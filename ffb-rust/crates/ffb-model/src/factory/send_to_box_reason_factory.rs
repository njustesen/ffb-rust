use crate::enums::SendToBoxReason;

/// 1:1 translation of com.fumbbl.ffb.factory.SendToBoxReasonFactory.
pub struct SendToBoxReasonFactory;

impl Default for SendToBoxReasonFactory {
    fn default() -> Self { SendToBoxReasonFactory }
}

impl SendToBoxReasonFactory {
    pub fn for_name(&self, name: &str) -> Option<SendToBoxReason> {
        SendToBoxReason::from_name(name)
    }

    pub fn initialize(&mut self) {}
}
