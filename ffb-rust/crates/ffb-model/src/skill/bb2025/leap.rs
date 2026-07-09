/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::Leap.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct Leap {
    pub base: Skill,
}

impl Leap {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Leap", SkillCategory::Agility, SkillUsageType::OncePerTurn);
        Self { base }
    }
}

impl Default for Leap {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Leap {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(Leap::new().get_name(), "Leap");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(Leap::new().get_category(), SkillCategory::Agility);
    }
}
