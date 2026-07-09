use crate::inducement::inducement_duration::InducementDuration;

/// Trait for prayer effects — 1:1 translation of Java Prayer interface.
pub trait Prayer {
    fn get_name(&self) -> &str;
    fn affects_both_teams(&self) -> bool;
    fn get_description(&self) -> &str;
    fn get_duration(&self) -> InducementDuration;
    fn event_message(&self) -> &str { "" }
    fn is_changing_player(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SimplePrayer;

    impl Prayer for SimplePrayer {
        fn get_name(&self) -> &str { "Test" }
        fn affects_both_teams(&self) -> bool { false }
        fn get_description(&self) -> &str { "A test prayer" }
        fn get_duration(&self) -> InducementDuration { InducementDuration::UNTIL_END_OF_HALF }
        fn is_changing_player(&self) -> bool { false }
    }

    #[test]
    fn test_default_event_message() {
        let p = SimplePrayer;
        assert_eq!(p.event_message(), "");
    }

    #[test]
    fn test_name_and_affects_both() {
        let p = SimplePrayer;
        assert_eq!(p.get_name(), "Test");
        assert!(!p.affects_both_teams());
    }
}
