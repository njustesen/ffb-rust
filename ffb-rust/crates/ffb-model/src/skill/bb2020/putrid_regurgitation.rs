/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::PutridRegurgitation.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct PutridRegurgitation {
    pub base: Skill,
}

impl PutridRegurgitation {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Putrid Regurgitation", SkillCategory::Trait, SkillUsageType::OncePerGame);
        Self { base }
    }
}

impl Default for PutridRegurgitation {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for PutridRegurgitation {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(PutridRegurgitation::new().get_name(), "Putrid Regurgitation");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(PutridRegurgitation::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn usage_type_is_correct() {
        assert_eq!(PutridRegurgitation::new().get_skill_usage_type(), SkillUsageType::OncePerGame);
    }
}
