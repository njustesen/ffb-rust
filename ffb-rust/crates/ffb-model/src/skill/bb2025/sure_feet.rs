/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::SureFeet.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct SureFeet {
    pub base: Skill,
}

impl SureFeet {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Sure Feet", SkillCategory::Agility, SkillUsageType::OncePerTurn);
        Self { base }
    }
}

impl Default for SureFeet {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for SureFeet {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(SureFeet::new().get_name(), "Sure Feet");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(SureFeet::new().get_category(), SkillCategory::Agility);
    }
}
