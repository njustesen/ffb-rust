use crate::model::PositionChoiceMode;

/// 1:1 translation of com.fumbbl.ffb.factory.PositionChoiceModeFactory (if exists).
pub struct PositionChoiceModeFactory;

impl Default for PositionChoiceModeFactory {
    fn default() -> Self { PositionChoiceModeFactory }
}

impl PositionChoiceModeFactory {
    pub fn for_name(&self, name: &str) -> Option<PositionChoiceMode> {
        PositionChoiceMode::for_name(name)
    }

    pub fn initialize(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_returns_known_mode() {
        let f = PositionChoiceModeFactory::default();
        assert_eq!(f.for_name("raiseDead"), Some(PositionChoiceMode::RAISE_DEAD));
    }

    #[test]
    fn for_name_unknown_returns_none() {
        assert_eq!(PositionChoiceModeFactory::default().for_name("invalid"), None);
    }

    #[test]
    fn initialize_does_not_panic() {
        let mut f = PositionChoiceModeFactory::default();
        f.initialize();
    }

    #[test]
    fn for_name_raise_dead_returns_variant() {
        // PositionChoiceMode has only one variant; verify it round-trips correctly
        assert_eq!(
            PositionChoiceModeFactory::default().for_name("raiseDead"),
            Some(PositionChoiceMode::RAISE_DEAD)
        );
    }

    #[test]
    fn for_name_empty_string_returns_none() {
        assert_eq!(PositionChoiceModeFactory::default().for_name(""), None);
    }
}
