/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::ConsummateProfessional.
// NOTE: Java also calls registerRerollSource(ReRolledActions.SINGLE_DIE, ReRollSources.CONSUMMATE_PROFESSIONAL);
// there is no live reroll-source lookup table in the Rust codebase to mirror this into (Skill::register_reroll_source
// is dead code), so this is left as a gap pending that infrastructure.
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
}
