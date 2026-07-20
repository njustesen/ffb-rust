/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::PumpUpTheCrowd.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct PumpUpTheCrowd {
    pub base: Skill,
}

impl PumpUpTheCrowd {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Pump Up The Crowd", SkillCategory::Trait, SkillUsageType::OncePerGame);
        Self { base }
    }
}

impl Default for PumpUpTheCrowd {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for PumpUpTheCrowd {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::SkillId;

    #[test]
    fn name_is_pump_up_the_crowd() {
        assert_eq!(PumpUpTheCrowd::new().get_name(), "Pump Up The Crowd");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(PumpUpTheCrowd::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn has_grants_team_re_roll_when_causing_cas_property() {
        assert!(SkillId::PumpUpTheCrowd.properties().contains(&"grantsTeamReRollWhenCausingCas"));
    }

    #[test]
    fn usage_type_is_correct() {
        assert_eq!(PumpUpTheCrowd::new().get_skill_usage_type(), SkillUsageType::OncePerGame);
    }
}
