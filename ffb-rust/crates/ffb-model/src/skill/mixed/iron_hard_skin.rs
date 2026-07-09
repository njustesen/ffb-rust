/// 1:1 translation of com.fumbbl.ffb.skill.mixed::IronHardSkin.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct IronHardSkin {
    pub base: Skill,
}

impl IronHardSkin {
    pub fn new() -> Self {
        let base = Skill::new("Iron Hard Skin", SkillCategory::Mutation);
        Self { base }
    }
}

impl Default for IronHardSkin {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for IronHardSkin {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(IronHardSkin::new().get_name(), "Iron Hard Skin"); }
    #[test]
    fn category_is_correct() { assert_eq!(IronHardSkin::new().get_category(), SkillCategory::Mutation); }
}
