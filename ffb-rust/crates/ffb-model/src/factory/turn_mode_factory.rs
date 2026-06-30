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
