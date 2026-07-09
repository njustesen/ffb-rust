/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::BrutalBlock.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct BrutalBlock {
    pub base: Skill,
}

impl BrutalBlock {
    pub fn new() -> Self {
        let base = Skill::new("Brutal Block", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for BrutalBlock {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for BrutalBlock {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(BrutalBlock::new().get_name(), "Brutal Block");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(BrutalBlock::new().get_category(), SkillCategory::Trait);
    }
}
