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
