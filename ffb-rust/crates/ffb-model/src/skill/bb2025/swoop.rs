/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::Swoop.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct Swoop {
    pub base: Skill,
}

impl Swoop {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Swoop", SkillCategory::Trait, SkillUsageType::OncePerTurnByTeamMate);
        Self { base }
    }
}

impl Default for Swoop {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Swoop {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(Swoop::new().get_name(), "Swoop");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(Swoop::new().get_category(), SkillCategory::Trait);
    }
}
