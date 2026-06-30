use crate::enums::ModelChangeId;

/// 1:1 translation of com.fumbbl.ffb.factory.ModelChangeIdFactory.
pub struct ModelChangeIdFactory;

impl Default for ModelChangeIdFactory {
    fn default() -> Self { ModelChangeIdFactory }
}

impl ModelChangeIdFactory {
    pub fn for_name(&self, name: &str) -> Option<ModelChangeId> {
        ModelChangeId::for_name(name)
    }

    pub fn initialize(&mut self) {}
}
