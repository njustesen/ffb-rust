/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::MaximumCarnage.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct MaximumCarnage {
    pub base: Skill,
}

impl MaximumCarnage {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Maximum Carnage", SkillCategory::Trait, SkillUsageType::OncePerGame);
        Self { base }
    }
}

impl Default for MaximumCarnage {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for MaximumCarnage {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(MaximumCarnage::new().get_name(), "Maximum Carnage"); }
    #[test]
    fn category_is_correct() { assert_eq!(MaximumCarnage::new().get_category(), SkillCategory::Trait); }
    #[test]
    fn usage_type_is_once_per_game() { assert_eq!(MaximumCarnage::new().get_skill_usage_type(), SkillUsageType::OncePerGame); }
}
