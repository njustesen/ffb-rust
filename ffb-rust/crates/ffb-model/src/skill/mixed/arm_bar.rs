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
    fn name_is_arm_bar() {
        assert_eq!(ArmBar::new().get_name(), "Arm Bar");
    }

    #[test]
    fn category_is_strength() {
        assert_eq!(ArmBar::new().get_category(), SkillCategory::Strength);
    }

    #[test]
    fn has_affects_either_armour_or_injury_on_dodge_property() {
        assert!(crate::enums::SkillId::ArmBar.properties().contains(&"affectsEitherArmourOrInjuryOnDodge"));
    }

    #[test]
    fn class_name_is_arm_bar() {
        assert_eq!(crate::enums::SkillId::ArmBar.class_name(), "ArmBar");
    }
}
