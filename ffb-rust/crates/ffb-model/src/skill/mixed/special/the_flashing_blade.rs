/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::TheFlashingBlade.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct TheFlashingBlade {
    pub base: Skill,
}

impl TheFlashingBlade {
    pub fn new() -> Self {
        let base = Skill::new("The Flashing Blade", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for TheFlashingBlade {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for TheFlashingBlade {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(TheFlashingBlade::new().get_name(), "The Flashing Blade"); }
    #[test]
    fn category_is_correct() { assert_eq!(TheFlashingBlade::new().get_category(), SkillCategory::Trait); }
}
