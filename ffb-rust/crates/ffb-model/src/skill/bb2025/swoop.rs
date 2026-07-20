/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::Swoop.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct Swoop {
    pub base: Skill,
}

impl Swoop {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Swoop", SkillCategory::Trait, SkillUsageType::OncePerTurnByTeamMate);
        Self { base }
    }
}

impl Default for Swoop {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Swoop {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_swoop() {
        assert_eq!(Swoop::new().get_name(), "Swoop");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(Swoop::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn has_ttm_scatters_in_single_direction_property() {
        assert!(crate::enums::SkillId::Swoop.properties().contains(&"ttmScattersInSingleDirection"));
    }

    #[test]
    fn has_swoop_reroll_source() {
        // Java: bb2025/Swoop.postConstruct registers ReRolledActions.RIGHT_STUFF →
        // ReRollSources.SWOOP; mirrored against the live SkillId::reroll_sources() table.
        assert!(crate::enums::SkillId::Swoop.reroll_sources().iter().any(|(a, _)| *a == "RIGHT_STUFF"));
    }
}
