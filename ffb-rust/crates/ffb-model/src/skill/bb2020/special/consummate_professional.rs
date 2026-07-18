/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::ConsummateProfessional.
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

    #[test]
    fn name_is_correct() {
        assert_eq!(ConsummateProfessional::new().get_name(), "Consummate Professional");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(ConsummateProfessional::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn usage_type_is_once_per_game() {
        assert_eq!(ConsummateProfessional::new().get_skill_usage_type(), SkillUsageType::OncePerGame);
    }
}
