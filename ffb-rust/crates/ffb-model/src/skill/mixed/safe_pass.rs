/// 1:1 translation of com.fumbbl.ffb.skill.mixed::SafePass.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct SafePass {
    pub base: Skill,
}

impl SafePass {
    pub fn new() -> Self {
        let base = Skill::new("Safe Pass", SkillCategory::Passing);
        Self { base }
    }
}

impl Default for SafePass {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for SafePass {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(SafePass::new().get_name(), "Safe Pass"); }
    #[test]
    fn category_is_correct() { assert_eq!(SafePass::new().get_category(), SkillCategory::Passing); }
}
