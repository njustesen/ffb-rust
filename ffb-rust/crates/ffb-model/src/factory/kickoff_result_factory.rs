use crate::enums::KickoffResult;

/// 1:1 translation of com.fumbbl.ffb.factory.KickoffResultFactory.
pub struct KickoffResultFactory;

impl Default for KickoffResultFactory {
    fn default() -> Self { Self }
}

impl KickoffResultFactory {
    pub fn for_name(&self, name: &str) -> Option<KickoffResult> {
        KickoffResult::from_name(name)
    }

    pub fn initialize(&mut self) {}
}
