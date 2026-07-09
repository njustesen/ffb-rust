/// 1:1 translation of com.fumbbl.ffb.skill.mixed::MyBall.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct MyBall {
    pub base: Skill,
}

impl MyBall {
    pub fn new() -> Self {
        let base = Skill::new("My Ball", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for MyBall {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for MyBall {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(MyBall::new().get_name(), "My Ball"); }
    #[test]
    fn category_is_correct() { assert_eq!(MyBall::new().get_category(), SkillCategory::Trait); }
}
