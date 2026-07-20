/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::LoneFouler.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct LoneFouler {
    pub base: Skill,
}

impl LoneFouler {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Lone Fouler", SkillCategory::Devious, SkillUsageType::OncePerTurn);
        Self { base }
    }
}

impl Default for LoneFouler {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for LoneFouler {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_lone_fouler() {
        assert_eq!(LoneFouler::new().get_name(), "Lone Fouler");
    }

    #[test]
    fn category_is_devious() {
        assert_eq!(LoneFouler::new().get_category(), SkillCategory::Devious);
    }

    #[test]
    fn skill_properties_are_not_null() {
        // Java: assertNotNull(skill.getSkillProperties()); the bb2025 Java postConstruct
        // registers no NamedProperties, so the live SkillId table must be empty here.
        assert!(crate::enums::SkillId::LoneFouler.properties().is_empty());
    }

    #[test]
    fn usage_type_is_once_per_turn() {
        assert_eq!(LoneFouler::new().skill_usage_type, SkillUsageType::OncePerTurn);
    }
}
