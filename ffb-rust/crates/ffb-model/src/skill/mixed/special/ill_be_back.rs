/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::IllBeBack.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct IllBeBack {
    pub base: Skill,
}

impl IllBeBack {
    pub fn new() -> Self {
        let base = Skill::new("I'll be back!", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for IllBeBack {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for IllBeBack {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(IllBeBack::new().get_name(), "I'll be back!"); }
    #[test]
    fn category_is_correct() { assert_eq!(IllBeBack::new().get_category(), SkillCategory::Trait); }
}
