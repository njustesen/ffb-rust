/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::Kick.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Kick {
    pub base: Skill,
}

impl Kick {
    pub fn new() -> Self {
        let base = Skill::new("Kick", SkillCategory::General);
        Self { base }
    }
}

impl Default for Kick {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Kick {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_kick() {
        assert_eq!(Kick::new().get_name(), "Kick");
    }

    #[test]
    fn category_is_general() {
        assert_eq!(Kick::new().get_category(), SkillCategory::General);
    }

    #[test]
    fn has_can_reduce_kick_distance_property() {
        assert!(crate::enums::SkillId::Kick.properties().contains(&"canReduceKickDistance"));
    }

    #[test]
    fn has_skill_properties_not_null() {
        // Java: assertNotNull(skill.getSkillProperties()); the bb2025 Java postConstruct
        // registers NamedProperties, so the live SkillId table must be populated.
        assert!(!crate::enums::SkillId::Kick.properties().is_empty());
    }

    #[test]
    fn has_kick_reroll_sources() {
        // Java: bb2025/Kick.postConstruct registers PUNT_DIRECTION and PUNT_DISTANCE
        // (both → ReRollSources.KICK); mirrored against the live SkillId::reroll_sources() table.
        let sources = crate::enums::SkillId::Kick.reroll_sources();
        assert!(sources.iter().any(|(a, _)| *a == "PUNT_DIRECTION"));
        assert!(sources.iter().any(|(a, _)| *a == "PUNT_DISTANCE"));
    }
}
