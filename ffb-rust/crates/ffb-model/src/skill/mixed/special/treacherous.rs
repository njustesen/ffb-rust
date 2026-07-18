/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::Treacherous.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct Treacherous {
    pub base: Skill,
}

impl Treacherous {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Treacherous", SkillCategory::Trait, SkillUsageType::OncePerGame);
        Self { base }
    }
}

impl Default for Treacherous {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Treacherous {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(Treacherous::new().get_name(), "Treacherous"); }
    #[test]
    fn category_is_correct() { assert_eq!(Treacherous::new().get_category(), SkillCategory::Trait); }
    #[test]
    fn usage_type_is_once_per_game() { assert_eq!(Treacherous::new().skill_usage_type, SkillUsageType::OncePerGame); }
}
