/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::SafeThrow.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct SafeThrow {
    pub base: Skill,
}

impl SafeThrow {
    pub fn new() -> Self {
        let base = Skill::new("Safe Throw", SkillCategory::Passing);
        Self { base }
    }
}

impl Default for SafeThrow {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for SafeThrow {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::SkillId;

    #[test]
    fn name_is_safe_throw() {
        assert_eq!(SafeThrow::new().get_name(), "Safe Throw");
    }

    #[test]
    fn category_is_passing() {
        assert_eq!(SafeThrow::new().get_category(), SkillCategory::Passing);
    }

    #[test]
    fn has_can_cancel_interceptions_property() {
        assert!(SkillId::SafeThrow.properties().contains(&"canCancelInterceptions"));
    }
}
