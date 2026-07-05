use crate::enums::SkillCategory;

/// 1:1 translation of com.fumbbl.ffb.factory.SkillCategoryFactory.
pub struct SkillCategoryFactory;

impl Default for SkillCategoryFactory {
    fn default() -> Self { SkillCategoryFactory }
}

impl SkillCategoryFactory {
    pub fn for_name(&self, name: &str) -> Option<SkillCategory> {
        SkillCategory::from_name(name)
    }

    pub fn initialize(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_returns_known_category() {
        assert_eq!(SkillCategoryFactory::default().for_name("General"), Some(SkillCategory::General));
        assert_eq!(SkillCategoryFactory::default().for_name("Agility"), Some(SkillCategory::Agility));
    }

    #[test]
    fn for_name_unknown_returns_none() {
        assert_eq!(SkillCategoryFactory::default().for_name("invalid"), None);
    }
}
