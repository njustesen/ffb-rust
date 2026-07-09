/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::BlackInk.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct BlackInk {
    pub base: Skill,
}

impl BlackInk {
    pub fn new() -> Self {
        let base = Skill::new("Black Ink", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for BlackInk {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for BlackInk {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(BlackInk::new().get_name(), "Black Ink"); }
    #[test]
    fn category_is_correct() { assert_eq!(BlackInk::new().get_category(), SkillCategory::Trait); }
}
