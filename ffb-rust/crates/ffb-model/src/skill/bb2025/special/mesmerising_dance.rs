/// 1:1 translation of com.fumbbl.ffb.skill.bb2025.special::MesmerisingDance.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct MesmerisingDance {
    pub base: Skill,
}

impl MesmerisingDance {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Mesmerising Dance", SkillCategory::Trait, SkillUsageType::OncePerHalf);
        Self { base }
    }
}

impl Default for MesmerisingDance {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for MesmerisingDance {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(MesmerisingDance::new().get_name(), "Mesmerising Dance");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(MesmerisingDance::new().get_category(), SkillCategory::Trait);
    }
}
