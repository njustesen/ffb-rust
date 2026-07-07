use crate::enums::TurnMode;

/// 1:1 translation of com.fumbbl.ffb.factory.TurnModeFactory.
pub struct TurnModeFactory;

impl Default for TurnModeFactory {
    fn default() -> Self { Self }
}

impl TurnModeFactory {
    pub fn for_name(&self, name: &str) -> Option<TurnMode> {
        TurnMode::from_name(name)
    }

    pub fn initialize(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_returns_known_mode() {
        let f = TurnModeFactory::default();
        assert_eq!(f.for_name("regular"), Some(TurnMode::Regular));
    }

    #[test]
    fn for_name_unknown_returns_none() {
        assert_eq!(TurnModeFactory::default().for_name("invalid"), None);
    }

    #[test]
    fn for_name_blitz_returns_blitz() {
        let f = TurnModeFactory::default();
        assert!(f.for_name("blitz").is_some());
    }

    #[test]
    fn initialize_does_not_panic() {
        let mut f = TurnModeFactory::default();
        f.initialize();
    }
}
