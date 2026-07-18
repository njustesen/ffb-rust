/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::Slayer.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct Slayer {
    pub base: Skill,
}

impl Slayer {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Slayer", SkillCategory::Trait, SkillUsageType::OncePerGame);
        Self { base }
    }
}

impl Default for Slayer {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Slayer {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(Slayer::new().get_name(), "Slayer"); }
    #[test]
    fn category_is_correct() { assert_eq!(Slayer::new().get_category(), SkillCategory::Trait); }
    #[test]
    fn usage_type_is_once_per_game() { assert_eq!(Slayer::new().skill_usage_type, SkillUsageType::OncePerGame); }
}
