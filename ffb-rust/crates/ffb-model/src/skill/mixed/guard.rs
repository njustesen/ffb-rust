/// 1:1 translation of com.fumbbl.ffb.skill.mixed::Guard.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Guard {
    pub base: Skill,
}

impl Guard {
    pub fn new() -> Self {
        let base = Skill::new("Guard", SkillCategory::Strength);
        Self { base }
    }
}

impl Default for Guard {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Guard {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(Guard::new().get_name(), "Guard"); }
    #[test]
    fn category_is_correct() { assert_eq!(Guard::new().get_category(), SkillCategory::Strength); }
}
