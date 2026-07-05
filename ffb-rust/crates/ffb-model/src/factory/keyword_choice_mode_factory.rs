use crate::model::KeywordChoiceMode;

/// 1:1 translation of com.fumbbl.ffb.factory.KeywordChoiceModeFactory (if exists).
pub struct KeywordChoiceModeFactory;

impl Default for KeywordChoiceModeFactory {
    fn default() -> Self { KeywordChoiceModeFactory }
}

impl KeywordChoiceModeFactory {
    pub fn for_name(&self, name: &str) -> Option<KeywordChoiceMode> {
        KeywordChoiceMode::for_name(name)
    }

    pub fn initialize(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_returns_known_mode() {
        assert!(KeywordChoiceModeFactory::default().for_name("gettingEven").is_some());
    }

    #[test]
    fn for_name_unknown_returns_none() {
        assert_eq!(KeywordChoiceModeFactory::default().for_name("invalid"), None);
    }
}
