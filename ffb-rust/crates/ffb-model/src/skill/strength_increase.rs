/// 1:1 translation of com.fumbbl.ffb.skill::StrengthIncrease.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct StrengthIncrease {
    pub base: Skill,
}

impl StrengthIncrease {
    pub fn new() -> Self {
        let base = Skill::new("+ST", SkillCategory::StatIncrease);
        Self { base }
    }
}

impl Default for StrengthIncrease {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for StrengthIncrease {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_plus_st() {
        assert_eq!(StrengthIncrease::new().get_name(), "+ST");
    }

    #[test]
    fn category_is_stat_increase() {
        assert_eq!(StrengthIncrease::new().get_category(), SkillCategory::StatIncrease);
    }

    #[test]
    fn skill_properties_are_not_null() {
        // Java asserts getSkillProperties() is not null; the live Rust mechanism
        // always returns a slice (empty here — StrengthIncrease registers no NamedProperties).
        assert!(crate::enums::SkillId::StrengthIncrease.properties().is_empty());
    }
}
