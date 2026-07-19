/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::BurstOfSpeed.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct BurstOfSpeed {
    pub base: Skill,
}

impl BurstOfSpeed {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Burst of Speed", SkillCategory::Trait, SkillUsageType::OncePerGame);
        Self { base }
    }
}

impl Default for BurstOfSpeed {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for BurstOfSpeed {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(BurstOfSpeed::new().get_name(), "Burst of Speed");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(BurstOfSpeed::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn usage_type_is_correct() {
        assert_eq!(BurstOfSpeed::new().get_skill_usage_type(), SkillUsageType::OncePerGame);
    }

    #[test]
    fn registers_named_property() {
        use crate::enums::SkillId;
        assert!(SkillId::BurstOfSpeed.properties().contains(&"canMakeAnExtraGfiOnce"));
    }
}
