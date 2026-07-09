/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::PassBlock.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct PassBlock {
    pub base: Skill,
}

impl PassBlock {
    pub fn new() -> Self {
        let base = Skill::new("Pass Block", SkillCategory::General);
        Self { base }
    }
}

impl Default for PassBlock {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for PassBlock {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(PassBlock::new().get_name(), "Pass Block");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(PassBlock::new().get_category(), SkillCategory::General);
    }
}
