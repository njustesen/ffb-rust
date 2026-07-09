/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::NoHands.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct NoHands {
    pub base: Skill,
}

impl NoHands {
    pub fn new() -> Self {
        let base = Skill::new("No Hands", SkillCategory::Extraordinary);
        Self { base }
    }
}

impl Default for NoHands {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for NoHands {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(NoHands::new().get_name(), "No Hands");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(NoHands::new().get_category(), SkillCategory::Extraordinary);
    }
}
