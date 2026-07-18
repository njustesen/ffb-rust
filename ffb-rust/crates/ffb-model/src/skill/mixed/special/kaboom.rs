/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::Kaboom.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct Kaboom {
    pub base: Skill,
}

impl Kaboom {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Kaboom!", SkillCategory::Trait, SkillUsageType::OncePerGame);
        Self { base }
    }
}

impl Default for Kaboom {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Kaboom {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(Kaboom::new().get_name(), "Kaboom!"); }
    #[test]
    fn category_is_correct() { assert_eq!(Kaboom::new().get_category(), SkillCategory::Trait); }
    #[test]
    fn usage_type_is_once_per_game() { assert_eq!(Kaboom::new().get_skill_usage_type(), SkillUsageType::OncePerGame); }
}
