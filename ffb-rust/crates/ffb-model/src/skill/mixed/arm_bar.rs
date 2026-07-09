/// 1:1 translation of com.fumbbl.ffb.skill.mixed::ArmBar.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct ArmBar {
    pub base: Skill,
}

impl ArmBar {
    pub fn new() -> Self {
        let base = Skill::new("Arm Bar", SkillCategory::Strength);
        Self { base }
    }
}

impl Default for ArmBar {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for ArmBar {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(ArmBar::new().get_name(), "Arm Bar"); }
    #[test]
    fn category_is_correct() { assert_eq!(ArmBar::new().get_category(), SkillCategory::Strength); }
}
