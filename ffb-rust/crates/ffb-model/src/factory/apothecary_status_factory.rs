use crate::enums::ApothecaryStatus;

/// 1:1 translation of com.fumbbl.ffb.factory.ApothecaryStatusFactory.
pub struct ApothecaryStatusFactory;

impl Default for ApothecaryStatusFactory {
    fn default() -> Self { Self }
}

impl ApothecaryStatusFactory {
    pub fn for_name(&self, name: &str) -> Option<ApothecaryStatus> {
        ApothecaryStatus::from_name(name)
    }

    pub fn initialize(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn for_name_returns_variant() {
        assert_eq!(ApothecaryStatusFactory.for_name("noApothecary"), Some(ApothecaryStatus::NoApothecary));
    }
    #[test]
    fn for_name_unknown_returns_none() {
        assert!(ApothecaryStatusFactory.for_name("UNKNOWN").is_none());
    }

    #[test]
    fn initialize_does_not_panic() {
        let mut f = ApothecaryStatusFactory;
        f.initialize();
    }

    #[test]
    fn for_name_a_second_known_variant() {
        assert_eq!(ApothecaryStatusFactory.for_name("useApothecary"), Some(ApothecaryStatus::UseApothecary));
    }

    #[test]
    fn for_name_empty_string_returns_none() {
        assert!(ApothecaryStatusFactory.for_name("").is_none());
    }
}
