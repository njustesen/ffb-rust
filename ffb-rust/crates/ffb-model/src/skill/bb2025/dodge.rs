/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::Dodge.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct Dodge {
    pub base: Skill,
}

impl Dodge {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Dodge", SkillCategory::Agility, SkillUsageType::OncePerTurn);
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
        // Java: bb2025/Dodge.postConstruct registers ReRolledActions.DODGE →
        // ReRollSources.DODGE; mirrored against the live SkillId::reroll_sources() table.
        assert!(crate::enums::SkillId::Dodge.reroll_sources().iter().any(|(a, _)| *a == "DODGE"));
    }
}
