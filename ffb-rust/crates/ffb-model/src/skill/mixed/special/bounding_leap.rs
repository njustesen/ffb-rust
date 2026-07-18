/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::BoundingLeap.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct BoundingLeap {
    pub base: Skill,
}

impl BoundingLeap {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Bounding Leap", SkillCategory::Trait, SkillUsageType::OncePerGame);
        Self { base }
    }
}

impl Default for BoundingLeap {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for BoundingLeap {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(BoundingLeap::new().get_name(), "Bounding Leap"); }
    #[test]
    fn category_is_correct() { assert_eq!(BoundingLeap::new().get_category(), SkillCategory::Trait); }
    #[test]
    fn usage_type_is_once_per_game() { assert_eq!(BoundingLeap::new().get_skill_usage_type(), SkillUsageType::OncePerGame); }
}
