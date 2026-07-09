/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::Ram.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Ram {
    pub base: Skill,
}

impl Ram {
    pub fn new() -> Self {
        let base = Skill::new("Ram", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for Ram {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Ram {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(Ram::new().get_name(), "Ram"); }
    #[test]
    fn category_is_correct() { assert_eq!(Ram::new().get_category(), SkillCategory::Trait); }
}
