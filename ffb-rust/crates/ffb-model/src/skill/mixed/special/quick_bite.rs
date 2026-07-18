/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::QuickBite.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct QuickBite {
    pub base: Skill,
}

impl QuickBite {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Quick Bite", SkillCategory::Trait, SkillUsageType::OncePerGame);
        Self { base }
    }
}

impl Default for QuickBite {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for QuickBite {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(QuickBite::new().get_name(), "Quick Bite"); }
    #[test]
    fn category_is_correct() { assert_eq!(QuickBite::new().get_category(), SkillCategory::Trait); }
    #[test]
    fn usage_type_is_once_per_game() { assert_eq!(QuickBite::new().skill_usage_type, SkillUsageType::OncePerGame); }
}
