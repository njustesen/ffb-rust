use crate::enums::PassResult;

/// 1:1 translation of com.fumbbl.ffb.factory.PassResultFactory.
pub struct PassResultFactory;

impl Default for PassResultFactory {
    fn default() -> Self { Self }
}

impl PassResultFactory {
    pub fn for_name(&self, name: &str) -> Option<PassResult> {
        PassResult::from_name(name)
    }

    pub fn initialize(&mut self) {}
}
