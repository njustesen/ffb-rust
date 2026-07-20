/// 1:1 translation of com.fumbbl.ffb.skill.bb2025.special::MesmerisingDance.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct MesmerisingDance {
    pub base: Skill,
}

impl MesmerisingDance {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Mesmerising Dance", SkillCategory::Trait, SkillUsageType::OncePerHalf);
        Self { base }
    }
}

impl Default for MesmerisingDance {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for MesmerisingDance {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_mesmerising_dance() {
        assert_eq!(MesmerisingDance::new().get_name(), "Mesmerising Dance");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(MesmerisingDance::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn skill_properties_are_not_null() {
        // Java: assertNotNull(skill.getSkillProperties()); the bb2025 Java postConstruct
        // registers no NamedProperties, so the live SkillId table must be empty here.
        assert!(crate::enums::SkillId::MesmerisingDance.properties().is_empty());
    }
}
