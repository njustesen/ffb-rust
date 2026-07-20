/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::AgilityIncrease.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct AgilityIncrease {
    pub base: Skill,
}

impl AgilityIncrease {
    pub fn new() -> Self {
        let base = Skill::new("+AG", SkillCategory::StatIncrease);
        Self { base }
    }
}

impl Default for AgilityIncrease {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for AgilityIncrease {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_plus_ag() {
        assert_eq!(AgilityIncrease::new().get_name(), "+AG");
    }

    #[test]
    fn category_is_stat_increase() {
        assert_eq!(AgilityIncrease::new().get_category(), SkillCategory::StatIncrease);
    }

    #[test]
    fn skill_properties_are_not_null() {
        // Java: assertNotNull(skill.getSkillProperties()); the bb2025 Java postConstruct
        // registers no NamedProperties, so the live SkillId table must be empty here.
        assert!(crate::enums::SkillId::AgilityIncrease.properties().is_empty());
    }

    #[test]
    fn class_name_is_agility_increase() {
        assert_eq!(crate::enums::SkillId::AgilityIncrease.class_name(), "AgilityIncrease");
    }
}
