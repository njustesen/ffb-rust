/// 1:1 translation of com.fumbbl.ffb.skill.bb2025.special::DwarvenScourge.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct DwarvenScourge {
    pub base: Skill,
}

impl DwarvenScourge {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Dwarven Scourge", SkillCategory::Trait, SkillUsageType::OncePerGame);
        Self { base }
    }
}

impl Default for DwarvenScourge {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for DwarvenScourge {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(DwarvenScourge::new().get_name(), "Dwarven Scourge");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(DwarvenScourge::new().get_category(), SkillCategory::Trait);
    }
}
