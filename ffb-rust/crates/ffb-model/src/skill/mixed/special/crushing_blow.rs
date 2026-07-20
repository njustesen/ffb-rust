/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::CrushingBlow.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct CrushingBlow {
    pub base: Skill,
}

impl CrushingBlow {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Crushing Blow", SkillCategory::Trait, SkillUsageType::OncePerGame);
        Self { base }
    }
}

impl Default for CrushingBlow {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for CrushingBlow {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_crushing_blow() {
        assert_eq!(CrushingBlow::new().get_name(), "Crushing Blow");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(CrushingBlow::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn has_skill_properties_not_null() {
        // Java: assertNotNull(skill.getSkillProperties()); the live Rust
        // mechanism is SkillId::properties(), whose slice always exists.
        let props: &'static [&'static str] = crate::enums::SkillId::CrushingBlow.properties();
        assert!(props.iter().all(|p| !p.is_empty()));
    }

    #[test]
    fn usage_type_is_once_per_game() {
        assert_eq!(CrushingBlow::new().get_skill_usage_type(), SkillUsageType::OncePerGame);
    }
}
