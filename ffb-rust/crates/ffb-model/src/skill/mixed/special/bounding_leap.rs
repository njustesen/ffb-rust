/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::BoundingLeap.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct BoundingLeap {
    pub base: Skill,
}

impl BoundingLeap {
    pub fn new() -> Self {
        let base = Skill::new("Bounding Leap", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for BoundingLeap {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for BoundingLeap {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(BoundingLeap::new().get_name(), "Bounding Leap"); }
    #[test]
    fn category_is_correct() { assert_eq!(BoundingLeap::new().get_category(), SkillCategory::Trait); }
}
