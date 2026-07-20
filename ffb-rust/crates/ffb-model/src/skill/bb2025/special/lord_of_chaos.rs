/// 1:1 translation of com.fumbbl.ffb.skill.bb2025.special::LordOfChaos.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct LordOfChaos {
    pub base: Skill,
}

impl LordOfChaos {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Lord of Chaos", SkillCategory::Trait, SkillUsageType::OncePerGame);
        Self { base }
    }
}

impl Default for LordOfChaos {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for LordOfChaos {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_lord_of_chaos() {
        assert_eq!(LordOfChaos::new().get_name(), "Lord of Chaos");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(LordOfChaos::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn has_lord_of_chaos_reroll_source() {
        // Java: bb2025/special/LordOfChaos.postConstruct registers ReRolledActions.SINGLE_BLOCK_DIE →
        // ReRollSources.LORD_OF_CHAOS; mirrored against the live SkillId::reroll_sources() table.
        assert!(crate::enums::SkillId::LordOfChaos.reroll_sources().iter().any(|(a, _)| *a == "SINGLE_BLOCK_DIE"));
    }
}
