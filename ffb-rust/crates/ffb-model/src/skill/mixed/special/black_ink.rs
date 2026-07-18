/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::BlackInk.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct BlackInk {
    pub base: Skill,
}

impl BlackInk {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Black Ink", SkillCategory::Trait, SkillUsageType::OncePerGame);
        Self { base }
    }
}

impl Default for BlackInk {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for BlackInk {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(BlackInk::new().get_name(), "Black Ink"); }
    #[test]
    fn category_is_correct() { assert_eq!(BlackInk::new().get_category(), SkillCategory::Trait); }
    #[test]
    fn usage_type_is_once_per_game() { assert_eq!(BlackInk::new().get_skill_usage_type(), SkillUsageType::OncePerGame); }
}
