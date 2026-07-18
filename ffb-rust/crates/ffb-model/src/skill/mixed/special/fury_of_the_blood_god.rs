/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::FuryOfTheBloodGod.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct FuryOfTheBloodGod {
    pub base: Skill,
}

impl FuryOfTheBloodGod {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Fury of the Blood God", SkillCategory::Trait, SkillUsageType::OncePerGame);
        Self { base }
    }
}

impl Default for FuryOfTheBloodGod {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for FuryOfTheBloodGod {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(FuryOfTheBloodGod::new().get_name(), "Fury of the Blood God"); }
    #[test]
    fn category_is_correct() { assert_eq!(FuryOfTheBloodGod::new().get_category(), SkillCategory::Trait); }
    #[test]
    fn usage_type_is_once_per_game() { assert_eq!(FuryOfTheBloodGod::new().get_skill_usage_type(), SkillUsageType::OncePerGame); }
}
