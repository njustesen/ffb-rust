/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::BlastIt.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct BlastIt {
    pub base: Skill,
}

impl BlastIt {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Blast It!", SkillCategory::Trait, SkillUsageType::OncePerGame);
        Self { base }
    }
}

impl Default for BlastIt {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for BlastIt {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(BlastIt::new().get_name(), "Blast It!");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(BlastIt::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn usage_type_is_once_per_game() {
        assert_eq!(BlastIt::new().get_skill_usage_type(), SkillUsageType::OncePerGame);
    }
}
