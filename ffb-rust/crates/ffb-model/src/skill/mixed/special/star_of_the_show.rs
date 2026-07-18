/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::StarOfTheShow.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct StarOfTheShow {
    pub base: Skill,
}

impl StarOfTheShow {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Star of the Show", SkillCategory::Trait, SkillUsageType::OncePerGame);
        Self { base }
    }
}

impl Default for StarOfTheShow {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for StarOfTheShow {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(StarOfTheShow::new().get_name(), "Star of the Show"); }
    #[test]
    fn category_is_correct() { assert_eq!(StarOfTheShow::new().get_category(), SkillCategory::Trait); }
    #[test]
    fn usage_type_is_once_per_game() { assert_eq!(StarOfTheShow::new().skill_usage_type, SkillUsageType::OncePerGame); }
}
