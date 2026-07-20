/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::OldPro.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct OldPro {
    pub base: Skill,
}

impl OldPro {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Old Pro", SkillCategory::Trait, SkillUsageType::OncePerGame);
        Self { base }
    }
}

impl Default for OldPro {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for OldPro {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_old_pro() {
        assert_eq!(OldPro::new().get_name(), "Old Pro");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(OldPro::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn has_skill_properties_not_null() {
        // Java: assertNotNull(skill.getSkillProperties()); the live Rust
        // mechanism is SkillId::properties(), whose slice always exists.
        let props: &'static [&'static str] = crate::enums::SkillId::OldPro.properties();
        assert!(props.iter().all(|p| !p.is_empty()));
    }

    #[test]
    fn usage_type_is_once_per_game() {
        assert_eq!(OldPro::new().get_skill_usage_type(), SkillUsageType::OncePerGame);
    }
}
