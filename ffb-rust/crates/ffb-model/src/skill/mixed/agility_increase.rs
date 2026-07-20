/// 1:1 translation of com.fumbbl.ffb.skill.mixed::AgilityIncrease.
/// Deferred: Java's `getCost(Player<?> player)` override (returns 40000) is not translated —
/// `Skill` has no `get_cost` method in Rust at all, and its Java implementation depends on
/// `Position::hasSkill`/`isDoubleCategory`, neither of which exist on the Rust `Position` model.
/// Adding this would require new cross-cutting infra beyond this file's scope.
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

    // Java: assertNotNull(skill.getSkillProperties()). The live Rust mechanism always
    // returns a valid slice; AgilityIncrease registers no NamedProperties, so the live
    // property table is empty.
    #[test]
    fn skill_properties_are_not_null() {
        assert!(crate::enums::SkillId::AgilityIncrease.properties().is_empty());
    }

    #[test]
    fn class_name_is_agility_increase() {
        assert_eq!(crate::enums::SkillId::AgilityIncrease.class_name(), "AgilityIncrease");
    }
}
