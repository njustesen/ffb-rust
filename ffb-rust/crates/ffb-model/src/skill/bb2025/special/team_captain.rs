/// 1:1 translation of com.fumbbl.ffb.skill.bb2025.special::TeamCaptain.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct TeamCaptain {
    pub base: Skill,
}

impl TeamCaptain {
    pub fn new() -> Self {
        let base = Skill::new("Team Captain", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for TeamCaptain {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for TeamCaptain {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(TeamCaptain::new().get_name(), "Team Captain");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(TeamCaptain::new().get_category(), SkillCategory::Trait);
    }
}
