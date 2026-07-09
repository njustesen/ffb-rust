/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::QuickBite.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct QuickBite {
    pub base: Skill,
}

impl QuickBite {
    pub fn new() -> Self {
        let base = Skill::new("Quick Bite", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for QuickBite {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for QuickBite {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(QuickBite::new().get_name(), "Quick Bite"); }
    #[test]
    fn category_is_correct() { assert_eq!(QuickBite::new().get_category(), SkillCategory::Trait); }
}
