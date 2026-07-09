/// 1:1 translation of com.fumbbl.ffb.model.skill::Skill (minimal fields for skill identity).
use crate::enums::SkillCategory;

pub struct Skill {
    pub name: String,
    pub category: SkillCategory,
}

impl Skill {
    pub fn new(name: &str, category: SkillCategory) -> Self {
        Self {
            name: name.to_string(),
            category,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_category(&self) -> SkillCategory {
        self.category
    }
}
