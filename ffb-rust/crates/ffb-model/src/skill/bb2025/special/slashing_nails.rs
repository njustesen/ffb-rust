/// 1:1 translation of com.fumbbl.ffb.skill.bb2025.special::SlashingNails.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct SlashingNails {
    pub base: Skill,
}

impl SlashingNails {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Slashing Nails", SkillCategory::Trait, SkillUsageType::OncePerHalf);
        Self { base }
    }
}

impl Default for SlashingNails {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for SlashingNails {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(SlashingNails::new().get_name(), "Slashing Nails");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(SlashingNails::new().get_category(), SkillCategory::Trait);
    }
}
