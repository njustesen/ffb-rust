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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_returns_known_type() {
        assert_eq!(ModelChangeDataTypeFactory::default().for_name("boolean"), Some(ModelChangeDataType::Boolean));
        assert_eq!(ModelChangeDataTypeFactory::default().for_name("integer"), Some(ModelChangeDataType::Integer));
    }

    #[test]
    fn for_name_unknown_returns_none() {
        assert_eq!(ModelChangeDataTypeFactory::default().for_name("invalid"), None);
    }

    #[test]
    fn initialize_does_not_panic() {
        let mut f = ModelChangeDataTypeFactory::default();
        f.initialize();
    }

    #[test]
    fn for_name_a_second_known_variant() {
        assert_eq!(ModelChangeDataTypeFactory::default().for_name("skill"), Some(ModelChangeDataType::Skill));
    }

    #[test]
    fn for_name_empty_string_returns_none() {
        assert_eq!(ModelChangeDataTypeFactory::default().for_name(""), None);
    }
}
