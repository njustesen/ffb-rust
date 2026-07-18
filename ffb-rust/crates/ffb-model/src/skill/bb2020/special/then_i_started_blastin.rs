/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::ThenIStartedBlastin.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct ThenIStartedBlastin {
    pub base: Skill,
}

impl ThenIStartedBlastin {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("\"Then I Started Blastin'!\"", SkillCategory::Trait, SkillUsageType::OncePerHalf);
        Self { base }
    }
}

impl Default for ThenIStartedBlastin {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for ThenIStartedBlastin {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(ThenIStartedBlastin::new().get_name(), "\"Then I Started Blastin'!\"");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(ThenIStartedBlastin::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn usage_type_is_once_per_half() {
        assert_eq!(ThenIStartedBlastin::new().get_skill_usage_type(), SkillUsageType::OncePerHalf);
    }
}
