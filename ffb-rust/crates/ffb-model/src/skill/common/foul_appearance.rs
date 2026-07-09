/// 1:1 translation of com.fumbbl.ffb.skill.common::FoulAppearance.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct FoulAppearance {
    pub base: Skill,
}

impl FoulAppearance {
    pub fn new() -> Self {
        let base = Skill::new("Foul Appearance", SkillCategory::Mutation);
        Self { base }
    }
}

impl Default for FoulAppearance {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for FoulAppearance {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(FoulAppearance::new().get_name(), "Foul Appearance"); }
    #[test]
    fn category_is_correct() { assert_eq!(FoulAppearance::new().get_category(), SkillCategory::Mutation); }
}
