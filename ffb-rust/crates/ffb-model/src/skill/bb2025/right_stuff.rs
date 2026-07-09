/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::RightStuff.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct RightStuff {
    pub base: Skill,
}

impl RightStuff {
    pub fn new() -> Self {
        let base = Skill::new("Right Stuff", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for RightStuff {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for RightStuff {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(RightStuff::new().get_name(), "Right Stuff");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(RightStuff::new().get_category(), SkillCategory::Trait);
    }
}
