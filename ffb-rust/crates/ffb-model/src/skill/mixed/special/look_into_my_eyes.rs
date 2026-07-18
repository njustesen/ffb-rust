/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::LookIntoMyEyes.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct LookIntoMyEyes {
    pub base: Skill,
}

impl LookIntoMyEyes {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Look Into My Eyes", SkillCategory::Trait, SkillUsageType::OncePerGame);
        Self { base }
    }
}

impl Default for LookIntoMyEyes {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for LookIntoMyEyes {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(LookIntoMyEyes::new().get_name(), "Look Into My Eyes"); }
    #[test]
    fn category_is_correct() { assert_eq!(LookIntoMyEyes::new().get_category(), SkillCategory::Trait); }
    #[test]
    fn usage_type_is_once_per_game() { assert_eq!(LookIntoMyEyes::new().get_skill_usage_type(), SkillUsageType::OncePerGame); }
}
