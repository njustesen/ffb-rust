/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::Leap.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Leap {
    pub base: Skill,
}

impl Leap {
    pub fn new() -> Self {
        // Java: `super("Leap", SkillCategory.AGILITY)` — the two-arg constructor
        // defaults to `SkillUsageType.REGULAR`. A prior translation incorrectly
        // used `SkillUsageType::OncePerTurn`.
        let base = Skill::new("Leap", SkillCategory::Agility);
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

    #[test]
    fn usage_type_is_regular() {
        // Java `super("Leap", SkillCategory.AGILITY)` uses the two-arg constructor,
        // which defaults to SkillUsageType.REGULAR (not ONCE_PER_TURN).
        use crate::enums::SkillUsageType;
        assert_eq!(Leap::new().get_skill_usage_type(), SkillUsageType::Regular);
    }
}
