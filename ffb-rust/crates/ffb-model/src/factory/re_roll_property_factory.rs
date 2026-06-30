use crate::enums::ReRollProperty;

/// 1:1 translation of com.fumbbl.ffb.factory.ReRollPropertyFactory.
pub struct ReRollPropertyFactory;

impl Default for ReRollPropertyFactory {
    fn default() -> Self { ReRollPropertyFactory }
}

impl ReRollPropertyFactory {
    pub fn for_name(&self, name: &str) -> Option<ReRollProperty> {
        ReRollProperty::from_name(name)
    }

    pub fn initialize(&mut self) {}
}
