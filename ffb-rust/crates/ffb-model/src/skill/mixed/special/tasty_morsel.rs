/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::TastyMorsel.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct TastyMorsel {
    pub base: Skill,
}

impl TastyMorsel {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Tasty Morsel", SkillCategory::Trait, SkillUsageType::OncePerGame);
        Self { base }
    }
}

impl Default for TastyMorsel {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for TastyMorsel {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_tasty_morsel() {
        assert_eq!(TastyMorsel::new().get_name(), "Tasty Morsel");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(TastyMorsel::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn has_skill_properties_not_null() {
        // Java: assertNotNull(skill.getSkillProperties()); properties() always returns a valid slice.
        let _properties: &'static [&'static str] = crate::enums::SkillId::TastyMorsel.properties();
    }

    #[test]
    fn usage_type_is_once_per_game() {
        assert_eq!(TastyMorsel::new().skill_usage_type, SkillUsageType::OncePerGame);
    }
}
