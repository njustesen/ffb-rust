/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::TwoForOne.
// NOTE: Java postConstruct also calls setEnhancements(new TemporaryEnhancements().withSkills(singleton(
// SkillClassWithValue(Loner.class, "2")))) so the surviving partner replaces Loner (4+) with Loner (2+).
// Skill::set_enhancements is dead code in this codebase (no live consumer), so this is not yet wired up.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct TwoForOne {
    pub base: Skill,
}

impl TwoForOne {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Two for One", SkillCategory::Trait, SkillUsageType::Special);
        Self { base }
    }
}

impl Default for TwoForOne {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for TwoForOne {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::SkillId;

    #[test]
    fn name_is_two_for_one() {
        assert_eq!(TwoForOne::new().get_name(), "Two for One");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(TwoForOne::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn has_reduces_loner_roll_if_partner_is_hurt_property() {
        assert!(SkillId::TwoForOne.properties().contains(&"reducesLonerRollIfPartnerIsHurt"));
    }

    #[test]
    fn usage_type_is_correct() {
        assert_eq!(TwoForOne::new().get_skill_usage_type(), SkillUsageType::Special);
    }
}
