/// 1:1 translation of com.fumbbl.ffb.model.skill::Skill (base fields only).
use crate::enums::SkillCategory;

#[derive(Debug, Clone)]
pub struct Skill {
    name: String,
    category: SkillCategory,
}

impl Skill {
    pub fn new(name: impl Into<String>, category: SkillCategory) -> Self {
        Self { name: name.into(), category }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_category(&self) -> SkillCategory {
        self.category
    }
}

impl Default for Skill {
    fn default() -> Self {
        Self::new("", SkillCategory::General)
    }
}
