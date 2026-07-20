/// 1:1 translation of com.fumbbl.ffb.skill.mixed::ArmourIncrease.
/// Deferred: Java's `getCost(Player<?> player)` override (returns 10000, overriding the base
/// `com.fumbbl.ffb.skill.ArmourIncrease`'s own cost logic) is not translated — `Skill` has no
/// `get_cost` method in Rust, and the base class's cost logic depends on
/// `Position::hasSkill`/`isDoubleCategory`, which don't exist on the Rust `Position` model.
/// Adding this would require new cross-cutting infra beyond this file's scope.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct ArmourIncrease {
    pub base: Skill,
}

impl ArmourIncrease {
    pub fn new() -> Self {
        let base = Skill::new("+AV", SkillCategory::StatIncrease);
        Self { base }
    }
}

impl Default for ArmourIncrease {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for ArmourIncrease {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_plus_av() {
        assert_eq!(ArmourIncrease::new().get_name(), "+AV");
    }

    #[test]
    fn category_is_stat_increase() {
        assert_eq!(ArmourIncrease::new().get_category(), SkillCategory::StatIncrease);
    }

    // Java: assertNotNull(skill.getSkillProperties()). The live Rust mechanism always
    // returns a valid slice; ArmourIncrease registers no NamedProperties, so the live
    // property table is empty.
    #[test]
    fn skill_properties_are_not_null() {
        assert!(crate::enums::SkillId::ArmourIncrease.properties().is_empty());
    }
}
