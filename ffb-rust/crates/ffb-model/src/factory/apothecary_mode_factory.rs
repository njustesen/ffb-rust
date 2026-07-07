use crate::enums::ApothecaryMode;

/// 1:1 translation of com.fumbbl.ffb.factory.ApothecaryModeFactory.
pub struct ApothecaryModeFactory;

impl Default for ApothecaryModeFactory {
    fn default() -> Self { Self }
}

impl ApothecaryModeFactory {
    pub fn for_name(&self, name: &str) -> Option<ApothecaryMode> {
        ApothecaryMode::from_name(name)
    }

    pub fn initialize(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn for_name_returns_variant() {
        let f = ApothecaryModeFactory;
        assert_eq!(f.for_name("attacker"), Some(ApothecaryMode::Attacker));
    }
    #[test]
    fn for_name_unknown_returns_none() {
        assert!(ApothecaryModeFactory.for_name("NOT_VALID").is_none());
    }

    #[test]
    fn initialize_does_not_panic() {
        let mut f = ApothecaryModeFactory;
        f.initialize();
    }

    #[test]
    fn for_name_a_second_known_variant() {
        let f = ApothecaryModeFactory;
        assert_eq!(f.for_name("defender"), Some(ApothecaryMode::Defender));
    }

    #[test]
    fn for_name_empty_string_returns_none() {
        assert!(ApothecaryModeFactory.for_name("").is_none());
    }
}
