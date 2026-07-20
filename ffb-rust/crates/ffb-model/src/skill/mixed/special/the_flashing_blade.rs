/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::TheFlashingBlade.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct TheFlashingBlade {
    pub base: Skill,
}

impl TheFlashingBlade {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("The Flashing Blade", SkillCategory::Trait, SkillUsageType::OncePerGame);
        Self { base }
    }
}

impl Default for TheFlashingBlade {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for TheFlashingBlade {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_the_flashing_blade() {
        assert_eq!(TheFlashingBlade::new().get_name(), "The Flashing Blade");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(TheFlashingBlade::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn has_skill_properties_not_null() {
        // Java: assertNotNull(skill.getSkillProperties()); properties() always returns a valid slice.
        let _properties: &'static [&'static str] = crate::enums::SkillId::TheFlashingBlade.properties();
    }

    #[test]
    fn usage_type_is_once_per_game() {
        assert_eq!(TheFlashingBlade::new().skill_usage_type, SkillUsageType::OncePerGame);
    }
}
