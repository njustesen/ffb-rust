/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::CatchOfTheDay.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct CatchOfTheDay {
    pub base: Skill,
}

impl CatchOfTheDay {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Catch of the Day", SkillCategory::Trait, SkillUsageType::OncePerHalf);
        Self { base }
    }
}

impl Default for CatchOfTheDay {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for CatchOfTheDay {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(CatchOfTheDay::new().get_name(), "Catch of the Day"); }
    #[test]
    fn category_is_correct() { assert_eq!(CatchOfTheDay::new().get_category(), SkillCategory::Trait); }
    #[test]
    fn usage_type_is_once_per_half() { assert_eq!(CatchOfTheDay::new().get_skill_usage_type(), SkillUsageType::OncePerHalf); }
}
