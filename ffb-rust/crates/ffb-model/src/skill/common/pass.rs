/// 1:1 translation of com.fumbbl.ffb.skill.common::Pass.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Pass {
    pub base: Skill,
}

impl Pass {
    pub fn new() -> Self {
        let base = Skill::new("Pass", SkillCategory::Passing);
        Self { base }
    }
}

impl Default for Pass {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Pass {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(Pass::new().get_name(), "Pass"); }
    #[test]
    fn category_is_correct() { assert_eq!(Pass::new().get_category(), SkillCategory::Passing); }
}
