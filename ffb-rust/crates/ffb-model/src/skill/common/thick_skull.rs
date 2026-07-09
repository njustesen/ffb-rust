/// 1:1 translation of com.fumbbl.ffb.skill.common::ThickSkull.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct ThickSkull {
    pub base: Skill,
}

impl ThickSkull {
    pub fn new() -> Self {
        let base = Skill::new("Thick Skull", SkillCategory::Strength);
        Self { base }
    }
}

impl Default for ThickSkull {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for ThickSkull {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(ThickSkull::new().get_name(), "Thick Skull"); }
    #[test]
    fn category_is_correct() { assert_eq!(ThickSkull::new().get_category(), SkillCategory::Strength); }
}
