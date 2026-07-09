/// 1:1 translation of com.fumbbl.ffb.model.skill::Skill (base class — name + category).
use crate::enums::SkillCategory;

pub struct Skill {
    name: String,
    category: SkillCategory,
}

impl Skill {
    pub fn new(name: impl Into<String>, category: SkillCategory) -> Self {
        Self {
            name: name.into(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skill_stores_name_and_category() {
        let s = Skill::new("Animosity", SkillCategory::Extraordinary);
        assert_eq!(s.get_name(), "Animosity");
        assert_eq!(s.get_category(), SkillCategory::Extraordinary);
    }
}
