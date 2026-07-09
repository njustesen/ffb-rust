/// 1:1 translation of com.fumbbl.ffb.skill.mixed::MultipleBlock.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct MultipleBlock {
    pub base: Skill,
}

impl MultipleBlock {
    pub fn new() -> Self {
        let base = Skill::new("Multiple Block", SkillCategory::Strength);
        Self { base }
    }
}

impl Default for MultipleBlock {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for MultipleBlock {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(MultipleBlock::new().get_name(), "Multiple Block"); }
    #[test]
    fn category_is_correct() { assert_eq!(MultipleBlock::new().get_category(), SkillCategory::Strength); }
}
