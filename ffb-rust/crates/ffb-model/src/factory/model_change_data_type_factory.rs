use crate::enums::ModelChangeDataType;

/// 1:1 translation of com.fumbbl.ffb.factory.ModelChangeDataTypeFactory.
pub struct ModelChangeDataTypeFactory;

impl Default for ModelChangeDataTypeFactory {
    fn default() -> Self { ModelChangeDataTypeFactory }
}

impl ModelChangeDataTypeFactory {
    pub fn for_name(&self, name: &str) -> Option<ModelChangeDataType> {
        ModelChangeDataType::for_name(name)
    }

    pub fn initialize(&mut self) {}
}
