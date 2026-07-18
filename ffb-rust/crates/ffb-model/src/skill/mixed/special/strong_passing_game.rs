/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::StrongPassingGame.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct StrongPassingGame {
    pub base: Skill,
}

impl StrongPassingGame {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Strong Passing Game", SkillCategory::Trait, SkillUsageType::OncePerGame);
        Self { base }
    }
}

impl Default for StrongPassingGame {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for StrongPassingGame {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(StrongPassingGame::new().get_name(), "Strong Passing Game"); }
    #[test]
    fn category_is_correct() { assert_eq!(StrongPassingGame::new().get_category(), SkillCategory::Trait); }
    #[test]
    fn usage_type_is_once_per_game() { assert_eq!(StrongPassingGame::new().skill_usage_type, SkillUsageType::OncePerGame); }
}
