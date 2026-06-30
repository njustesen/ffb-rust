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
