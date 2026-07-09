/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::BreatheFire.
use crate::model::skill::skill::Skill;
use crate::enums::{DeclareCondition, SkillCategory};

pub struct BreatheFire {
    pub base: Skill,
}

impl BreatheFire {
    pub fn new() -> Self {
        let mut base = Skill::new("Breathe Fire", SkillCategory::Trait);
        base.declare_condition = DeclareCondition::Standing;
        Self { base }
    }
}

impl Default for BreatheFire {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for BreatheFire {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(BreatheFire::new().get_name(), "Breathe Fire");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(BreatheFire::new().get_category(), SkillCategory::Trait);
    }
}
