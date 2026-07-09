/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::MaximumCarnage.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct MaximumCarnage {
    pub base: Skill,
}

impl MaximumCarnage {
    pub fn new() -> Self {
        let base = Skill::new("Maximum Carnage", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for MaximumCarnage {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for MaximumCarnage {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(MaximumCarnage::new().get_name(), "Maximum Carnage"); }
    #[test]
    fn category_is_correct() { assert_eq!(MaximumCarnage::new().get_category(), SkillCategory::Trait); }
}
