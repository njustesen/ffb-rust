/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::WhirlingDervish.
// Java postConstruct calls registerRerollSource(ReRolledActions.DIRECTION, ReRollSources.
// WHIRLING_DERVISH); mirrored in the live `SkillId::reroll_sources()` table.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct WhirlingDervish {
    pub base: Skill,
}

impl WhirlingDervish {
    pub fn new() -> Self {
        let base = Skill::new("Whirling Dervish", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for WhirlingDervish {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for WhirlingDervish {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::SkillId;

    #[test]
    fn name_is_whirling_dervish() {
        assert_eq!(WhirlingDervish::new().get_name(), "Whirling Dervish");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(WhirlingDervish::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn skill_properties_are_not_null() {
        // Java: assertNotNull(skill.getSkillProperties()); the live Rust property table
        // is SkillId::WhirlingDervish.properties(), which always returns a valid slice.
        assert!(SkillId::WhirlingDervish.properties().iter().all(|p| !p.is_empty()));
    }

    #[test]
    fn has_whirling_dervish_reroll_source() {
        // Java: bb2020/special/WhirlingDervish.postConstruct registers ReRolledActions.DIRECTION →
        // ReRollSources.WHIRLING_DERVISH; mirrored against the live SkillId::reroll_sources() table.
        assert!(SkillId::WhirlingDervish.reroll_sources().iter().any(|(a, _)| *a == "DIRECTION"));
    }
}
