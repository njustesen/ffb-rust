/// 1:1 translation of com.fumbbl.ffb.skill.common::Block.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Block {
    pub base: Skill,
}

impl Block {
    pub fn new() -> Self {
        let base = Skill::new("Block", SkillCategory::General);
        Self { base }
    }
}

impl Default for Block {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Block {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(Block::new().get_name(), "Block"); }
    #[test]
    fn category_is_correct() { assert_eq!(Block::new().get_category(), SkillCategory::General); }
}
