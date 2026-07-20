/// 1:1 translation of com.fumbbl.ffb.skill.bb2025.special::TheBallista.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct TheBallista {
    pub base: Skill,
}

impl TheBallista {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("The Ballista", SkillCategory::Trait, SkillUsageType::OncePerGame);
        Self { base }
    }
}

impl Default for TheBallista {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for TheBallista {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_the_ballista() {
        assert_eq!(TheBallista::new().get_name(), "The Ballista");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(TheBallista::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn skill_properties_are_not_null() {
        // Java: assertNotNull(skill.getSkillProperties()); the bb2025 Java postConstruct
        // registers no NamedProperties, so the live SkillId table must be empty here.
        assert!(crate::enums::SkillId::TheBallista.properties().is_empty());
    }
}
