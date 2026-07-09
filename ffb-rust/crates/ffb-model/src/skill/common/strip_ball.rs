/// 1:1 translation of com.fumbbl.ffb.skill.common::StripBall.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct StripBall {
    pub base: Skill,
}

impl StripBall {
    pub fn new() -> Self {
        let base = Skill::new("Strip Ball", SkillCategory::General);
        Self { base }
    }
}

impl Default for StripBall {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for StripBall {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(StripBall::new().get_name(), "Strip Ball"); }
    #[test]
    fn category_is_correct() { assert_eq!(StripBall::new().get_category(), SkillCategory::General); }
}
