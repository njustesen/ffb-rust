/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::PassingIncrease.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct PassingIncrease {
    pub base: Skill,
}

impl PassingIncrease {
    pub fn new() -> Self {
        let base = Skill::new("+PA", SkillCategory::StatIncrease);
        Self { base }
    }
}

impl Default for PassingIncrease {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for PassingIncrease {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_plus_pa() {
        assert_eq!(PassingIncrease::new().get_name(), "+PA");
    }

    #[test]
    fn category_is_stat_increase() {
        assert_eq!(PassingIncrease::new().get_category(), SkillCategory::StatIncrease);
    }

    #[test]
    fn skill_properties_are_not_null() {
        // Java: assertNotNull(skill.getSkillProperties()); the bb2025 Java postConstruct
        // registers no NamedProperties, so the live SkillId table must be empty here.
        assert!(crate::enums::SkillId::PassingIncrease.properties().is_empty());
    }
}
