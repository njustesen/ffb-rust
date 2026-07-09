/// 1:1 translation of com.fumbbl.ffb.skill.common::HailMaryPass.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct HailMaryPass {
    pub base: Skill,
}

impl HailMaryPass {
    pub fn new() -> Self {
        let base = Skill::new("Hail Mary Pass", SkillCategory::Passing);
        Self { base }
    }
}

impl Default for HailMaryPass {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for HailMaryPass {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(HailMaryPass::new().get_name(), "Hail Mary Pass"); }
    #[test]
    fn category_is_correct() { assert_eq!(HailMaryPass::new().get_category(), SkillCategory::Passing); }
}
