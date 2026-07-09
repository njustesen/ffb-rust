/// 1:1 translation of com.fumbbl.ffb.skill.mixed::OnTheBall.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct OnTheBall {
    pub base: Skill,
}

impl OnTheBall {
    pub fn new() -> Self {
        let base = Skill::new("On The Ball", SkillCategory::Passing);
        Self { base }
    }
}

impl Default for OnTheBall {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for OnTheBall {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(OnTheBall::new().get_name(), "On The Ball"); }
    #[test]
    fn category_is_correct() { assert_eq!(OnTheBall::new().get_category(), SkillCategory::Passing); }
}
