/// 1:1 translation of com.fumbbl.ffb.skill.common::Tentacles.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Tentacles {
    pub base: Skill,
}

impl Tentacles {
    pub fn new() -> Self {
        let base = Skill::new("Tentacles", SkillCategory::Mutation);
        Self { base }
    }
}

impl Default for Tentacles {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Tentacles {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(Tentacles::new().get_name(), "Tentacles"); }
    #[test]
    fn category_is_correct() { assert_eq!(Tentacles::new().get_category(), SkillCategory::Mutation); }
}
