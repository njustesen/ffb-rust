/// 1:1 translation of com.fumbbl.ffb.skill.common::SureHands.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct SureHands {
    pub base: Skill,
}

impl SureHands {
    pub fn new() -> Self {
        let base = Skill::new("Sure Hands", SkillCategory::General);
        Self { base }
    }
}

impl Default for SureHands {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for SureHands {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(SureHands::new().get_name(), "Sure Hands"); }
    #[test]
    fn category_is_correct() { assert_eq!(SureHands::new().get_category(), SkillCategory::General); }
}
