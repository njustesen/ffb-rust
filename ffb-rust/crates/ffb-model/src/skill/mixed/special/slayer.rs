/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::Slayer.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct Slayer {
    pub base: Skill,
}

impl Slayer {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Slayer", SkillCategory::Trait, SkillUsageType::OncePerGame);
        Self { base }
    }
}

impl Default for Slayer {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Slayer {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_slayer() {
        assert_eq!(Slayer::new().get_name(), "Slayer");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(Slayer::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn skill_properties_are_not_null() {
        // Java: assertNotNull(skill.getSkillProperties()); properties() always returns a valid slice.
        let _properties: &'static [&'static str] = crate::enums::SkillId::Slayer.properties();
    }

    #[test]
    fn class_name_is_slayer() {
        // Java: assertEquals("Slayer", skill.getClass().getSimpleName());
        assert!(std::any::type_name::<Slayer>().ends_with("::Slayer"));
    }

    #[test]
    fn usage_type_is_once_per_game() {
        assert_eq!(Slayer::new().skill_usage_type, SkillUsageType::OncePerGame);
    }
}
