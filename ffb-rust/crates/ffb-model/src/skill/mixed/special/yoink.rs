/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::Yoink.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Yoink {
    pub base: Skill,
}

impl Yoink {
    pub fn new() -> Self {
        let base = Skill::new("Yoink!", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for Yoink {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Yoink {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(Yoink::new().get_name(), "Yoink!"); }
    #[test]
    fn category_is_correct() { assert_eq!(Yoink::new().get_category(), SkillCategory::Trait); }
}
