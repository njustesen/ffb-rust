/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::Yoink.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct Yoink {
    pub base: Skill,
}

impl Yoink {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Yoink!", SkillCategory::Trait, SkillUsageType::OncePerGame);
        Self { base }
    }
}

impl Default for Yoink {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Yoink {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_yoink() {
        assert_eq!(Yoink::new().get_name(), "Yoink!");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(Yoink::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn has_skill_properties_not_null() {
        // Java: assertNotNull(skill.getSkillProperties()); properties() always returns a valid slice.
        let _properties: &'static [&'static str] = crate::enums::SkillId::Yoink.properties();
    }

    #[test]
    fn usage_type_is_once_per_game() {
        assert_eq!(Yoink::new().skill_usage_type, SkillUsageType::OncePerGame);
    }
}
