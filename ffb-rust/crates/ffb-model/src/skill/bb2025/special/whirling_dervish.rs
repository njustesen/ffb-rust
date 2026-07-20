/// 1:1 translation of com.fumbbl.ffb.skill.bb2025.special::WhirlingDervish.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct WhirlingDervish {
    pub base: Skill,
}

impl WhirlingDervish {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Whirling Dervish", SkillCategory::Trait, SkillUsageType::OncePerTurn);
        Self { base }
    }
}

impl Default for WhirlingDervish {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for WhirlingDervish {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_whirling_dervish() {
        assert_eq!(WhirlingDervish::new().get_name(), "Whirling Dervish");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(WhirlingDervish::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn skill_properties_are_not_null() {
        // Java: assertNotNull(skill.getSkillProperties()); the bb2025 Java postConstruct
        // registers no NamedProperties, so the live SkillId table must be empty here.
        assert!(crate::enums::SkillId::WhirlingDervish.properties().is_empty());
    }
}
