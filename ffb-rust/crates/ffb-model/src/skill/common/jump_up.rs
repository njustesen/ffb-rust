/// 1:1 translation of com.fumbbl.ffb.skill.common::JumpUp.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct JumpUp {
    pub base: Skill,
}

impl JumpUp {
    pub fn new() -> Self {
        let base = Skill::new("Jump Up", SkillCategory::Agility);
        Self { base }
    }
}

impl Default for JumpUp {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for JumpUp {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(JumpUp::new().get_name(), "Jump Up"); }
    #[test]
    fn category_is_correct() { assert_eq!(JumpUp::new().get_category(), SkillCategory::Agility); }
}
