/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::NoBall.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct NoBall {
    pub base: Skill,
}

impl NoBall {
    pub fn new() -> Self {
        let base = Skill::new("No Ball", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for NoBall {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for NoBall {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(NoBall::new().get_name(), "No Ball");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(NoBall::new().get_category(), SkillCategory::Trait);
    }
}
