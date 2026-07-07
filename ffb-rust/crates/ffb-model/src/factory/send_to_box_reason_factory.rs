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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_returns_known_reason() {
        assert_eq!(SendToBoxReasonFactory::default().for_name("mng"), Some(SendToBoxReason::Mng));
        assert_eq!(SendToBoxReasonFactory::default().for_name("foulBan"), Some(SendToBoxReason::FoulBan));
    }

    #[test]
    fn for_name_unknown_returns_none() {
        assert_eq!(SendToBoxReasonFactory::default().for_name("invalid"), None);
    }

    #[test]
    fn for_name_second_known_variant() {
        let f = SendToBoxReasonFactory::default();
        assert!(f.for_name("foulBan").is_some());
    }

    #[test]
    fn initialize_does_not_panic() {
        let mut f = SendToBoxReasonFactory::default();
        f.initialize();
    }
}
