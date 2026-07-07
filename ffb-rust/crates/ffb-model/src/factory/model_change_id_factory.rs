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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_returns_known_id() {
        assert_eq!(
            ModelChangeIdFactory::default().for_name("actingPlayerMarkSkillUsed"),
            Some(ModelChangeId::ActingPlayerMarkSkillUsed)
        );
    }

    #[test]
    fn for_name_unknown_returns_none() {
        assert_eq!(ModelChangeIdFactory::default().for_name("invalid"), None);
    }

    #[test]
    fn initialize_does_not_panic() {
        let mut f = ModelChangeIdFactory::default();
        f.initialize();
    }

    #[test]
    fn for_name_a_second_known_variant() {
        assert_eq!(
            ModelChangeIdFactory::default().for_name("fieldModelSetWeather"),
            Some(ModelChangeId::FieldModelSetWeather)
        );
    }

    #[test]
    fn for_name_empty_string_returns_none() {
        assert_eq!(ModelChangeIdFactory::default().for_name(""), None);
    }
}
