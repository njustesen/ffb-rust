use crate::model::PushbackMode;

/// 1:1 translation of com.fumbbl.ffb.factory.PushbackModeFactory.
pub struct PushbackModeFactory;

impl Default for PushbackModeFactory {
    fn default() -> Self { PushbackModeFactory }
}

impl PushbackModeFactory {
    pub fn for_name(&self, name: &str) -> Option<PushbackMode> {
        PushbackMode::for_name(name)
    }

    pub fn initialize(&mut self) {}
}
