/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::ViolentInnovator.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct ViolentInnovator {
    pub base: Skill,
}

impl ViolentInnovator {
    pub fn new() -> Self {
        let base = Skill::new("Violent Innovator", SkillCategory::Devious);
        Self { base }
    }
}

impl Default for ViolentInnovator {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for ViolentInnovator {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(ViolentInnovator::new().get_name(), "Violent Innovator");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(ViolentInnovator::new().get_category(), SkillCategory::Devious);
    }
}
