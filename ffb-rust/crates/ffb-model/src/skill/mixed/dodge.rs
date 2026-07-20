/// 1:1 translation of com.fumbbl.ffb.skill.mixed::Dodge.
/// Java's `postConstruct()` registrations live in the static tables:
/// properties (canRerollDodge, ignoreDefenderStumblesResult) in `SkillId::properties()`,
/// the DODGE reroll source in `SkillId::reroll_sources()`.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Dodge {
    pub base: Skill,
}

impl Dodge {
    pub fn new() -> Self {
        let base = Skill::new("Dodge", SkillCategory::Agility);
        Self { base }
    }
}

impl Default for Dodge {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Dodge {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_dodge() {
        assert_eq!(Dodge::new().get_name(), "Dodge");
    }

    #[test]
    fn category_is_agility() {
        assert_eq!(Dodge::new().get_category(), SkillCategory::Agility);
    }

    #[test]
    fn has_ignore_defender_stumbles_result_property() {
        assert!(crate::enums::SkillId::Dodge.properties().contains(&"ignoreDefenderStumblesResult"));
    }

    #[test]
    fn does_not_have_force_followup_property() {
        assert!(!crate::enums::SkillId::Dodge.properties().contains(&"forceFollowup"));
    }

    #[test]
    fn has_dodge_reroll_source() {
        // Java: assertNotNull(skill.getRerollSource(ReRolledActions.DODGE)) —
        // mirrored against the live SkillId::reroll_sources() table.
        assert!(crate::enums::SkillId::Dodge.reroll_sources().iter().any(|(a, _)| *a == "DODGE"));
    }
}
