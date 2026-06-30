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
