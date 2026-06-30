use crate::model::CatchScatterThrowInMode;

/// 1:1 translation of com.fumbbl.ffb.factory.CatchScatterThrowInModeFactory.
pub struct CatchScatterThrowInModeFactory;

impl Default for CatchScatterThrowInModeFactory {
    fn default() -> Self { CatchScatterThrowInModeFactory }
}

impl CatchScatterThrowInModeFactory {
    pub fn for_name(&self, name: &str) -> Option<CatchScatterThrowInMode> {
        CatchScatterThrowInMode::for_name(name)
    }

    pub fn initialize(&mut self) {}
}
