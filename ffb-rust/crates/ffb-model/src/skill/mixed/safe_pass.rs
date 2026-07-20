/// 1:1 translation of com.fumbbl.ffb.skill.mixed::SafePass.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct SafePass {
    pub base: Skill,
}

impl SafePass {
    pub fn new() -> Self {
        let base = Skill::new("Safe Pass", SkillCategory::Passing);
        Self { base }
    }
}

impl Default for SafePass {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for SafePass {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mirrors ffb-java SafePassSkillTest.

    #[test]
    fn name_is_safe_pass() {
        assert_eq!(SafePass::new().get_name(), "Safe Pass");
    }

    #[test]
    fn category_is_passing() {
        assert_eq!(SafePass::new().get_category(), SkillCategory::Passing);
    }

    #[test]
    fn has_dont_drop_fumbles_property() {
        assert!(crate::enums::SkillId::SafePass.properties().contains(&"dontDropFumbles"));
    }

    #[test]
    fn class_name_is_safe_pass() {
        assert_eq!(crate::enums::SkillId::SafePass.class_name(), "SafePass");
    }
}
