/// 1:1 translation of com.fumbbl.ffb.skill.bb2025.special::KrumpAndSmash.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct KrumpAndSmash {
    pub base: Skill,
}

impl KrumpAndSmash {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Krump and Smash", SkillCategory::Trait, SkillUsageType::OncePerGame);
        Self { base }
    }
}

impl Default for KrumpAndSmash {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for KrumpAndSmash {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_krump_and_smash() {
        assert_eq!(KrumpAndSmash::new().get_name(), "Krump and Smash");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(KrumpAndSmash::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn skill_properties_are_not_null() {
        // Java: assertNotNull(skill.getSkillProperties()); the bb2025 Java postConstruct
        // registers no NamedProperties, so the live SkillId table must be empty here.
        assert!(crate::enums::SkillId::KrumpAndSmash.properties().is_empty());
    }
}
