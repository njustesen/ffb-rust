/// 1:1 translation of com.fumbbl.ffb.skill.bb2025.special::MasterAssassin.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct MasterAssassin {
    pub base: Skill,
}

impl MasterAssassin {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Master Assassin", SkillCategory::Trait, SkillUsageType::OncePerGame);
        Self { base }
    }
}

impl Default for MasterAssassin {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for MasterAssassin {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_master_assassin() {
        assert_eq!(MasterAssassin::new().get_name(), "Master Assassin");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(MasterAssassin::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn skill_properties_are_not_null() {
        // Java: assertNotNull(skill.getSkillProperties()); the bb2025 Java postConstruct
        // registers no NamedProperties, so the live SkillId table must be empty here.
        assert!(crate::enums::SkillId::MasterAssassin.properties().is_empty());
    }
}
