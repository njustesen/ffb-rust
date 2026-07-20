/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::ConsummateProfessional.
// Java also calls registerRerollSource(ReRolledActions.SINGLE_DIE, ReRollSources.CONSUMMATE_PROFESSIONAL);
// mirrored in the live `SkillId::reroll_sources()` table.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct ConsummateProfessional {
    pub base: Skill,
}

impl ConsummateProfessional {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Consummate Professional", SkillCategory::Trait, SkillUsageType::OncePerGame);
        Self { base }
    }
}

impl Default for ConsummateProfessional {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for ConsummateProfessional {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::SkillId;

    #[test]
    fn name_is_consummate_professional() {
        assert_eq!(ConsummateProfessional::new().get_name(), "Consummate Professional");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(ConsummateProfessional::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn has_can_reroll_single_die_once_per_period_property() {
        assert!(SkillId::ConsummateProfessional.properties().contains(&"canRerollSingleDieOncePerPeriod"));
    }

    #[test]
    fn usage_type_is_correct() {
        assert_eq!(ConsummateProfessional::new().get_skill_usage_type(), SkillUsageType::OncePerGame);
    }

    #[test]
    fn has_consummate_professional_reroll_source() {
        // Java: bb2020/special/ConsummateProfessional.postConstruct registers ReRolledActions.SINGLE_DIE →
        // ReRollSources.CONSUMMATE_PROFESSIONAL; mirrored against the live SkillId::reroll_sources() table.
        assert!(SkillId::ConsummateProfessional.reroll_sources().iter().any(|(a, _)| *a == "SINGLE_DIE"));
    }
}
