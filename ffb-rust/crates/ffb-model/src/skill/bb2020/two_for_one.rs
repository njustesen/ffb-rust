/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::TwoForOne.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct TwoForOne {
    pub base: Skill,
}

impl TwoForOne {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Two for One", SkillCategory::Trait, SkillUsageType::Special);
        Self { base }
    }
}

impl Default for TwoForOne {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for TwoForOne {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(TwoForOne::new().get_name(), "Two for One");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(TwoForOne::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn usage_type_is_correct() {
        assert_eq!(TwoForOne::new().get_skill_usage_type(), SkillUsageType::Special);
    }
}
