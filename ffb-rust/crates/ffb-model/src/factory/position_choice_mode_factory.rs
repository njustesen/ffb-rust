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
