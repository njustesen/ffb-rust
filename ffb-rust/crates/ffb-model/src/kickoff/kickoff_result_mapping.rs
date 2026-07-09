use crate::kickoff::KickoffEventKind;

/// 1:1 translation of `com.fumbbl.ffb.kickoff.KickoffResultMapping`.
///
/// Abstract base that maps a 2d6 roll to the edition-specific kickoff result.
/// Rust: use edition-specific functions from kickoff::mod.
pub trait KickoffResultMapping {
    /// Java: `getKey()` — returns the class simple name as a string key.
    fn get_key(&self) -> &str;

    /// Java: `getResult(int roll)` — map a 2d6 roll to its kickoff event.
    fn get_result(&self, roll: i32) -> Option<KickoffEventKind>;

    /// Java: `getValues()` — all kickoff events in this edition.
    fn get_values(&self) -> Vec<KickoffEventKind>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::Rules;
    use crate::kickoff::{kickoff_event_bb2025, KickoffEventKind};

    struct TestMapping;
    impl KickoffResultMapping for TestMapping {
        fn get_key(&self) -> &str { "TestMapping" }
        fn get_result(&self, roll: i32) -> Option<KickoffEventKind> {
            kickoff_event_bb2025(roll)
        }
        fn get_values(&self) -> Vec<KickoffEventKind> {
            (2..=12).filter_map(|r| kickoff_event_bb2025(r)).collect()
        }
    }

    #[test]
    fn mapping_key_is_non_empty() {
        let m = TestMapping;
        assert!(!m.get_key().is_empty());
    }

    #[test]
    fn valid_roll_returns_some() {
        let m = TestMapping;
        assert!(m.get_result(7).is_some());
    }

    #[test]
    fn invalid_roll_returns_none() {
        let m = TestMapping;
        assert!(m.get_result(1).is_none());
        assert!(m.get_result(13).is_none());
    }

    #[test]
    fn values_has_eleven_entries() {
        let m = TestMapping;
        assert_eq!(m.get_values().len(), 11);
    }
}
