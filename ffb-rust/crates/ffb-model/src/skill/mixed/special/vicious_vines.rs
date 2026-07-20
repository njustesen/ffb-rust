/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::ViciousVines.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct ViciousVines {
    pub base: Skill,
}

impl ViciousVines {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Vicious Vines", SkillCategory::Trait, SkillUsageType::OncePerHalf);
        Self { base }
    }
}

impl Default for ViciousVines {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for ViciousVines {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_vicious_vines() {
        assert_eq!(ViciousVines::new().get_name(), "Vicious Vines");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(ViciousVines::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn has_skill_properties_not_null() {
        // Java: assertNotNull(skill.getSkillProperties()); properties() always returns a valid slice.
        let _properties: &'static [&'static str] = crate::enums::SkillId::ViciousVines.properties();
    }

    #[test]
    fn usage_type_is_once_per_half() {
        assert_eq!(ViciousVines::new().skill_usage_type, SkillUsageType::OncePerHalf);
    }
}
