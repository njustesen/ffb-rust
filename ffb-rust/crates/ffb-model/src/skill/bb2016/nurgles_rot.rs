/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::NurglesRot.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct NurglesRot {
    pub base: Skill,
}

impl NurglesRot {
    pub fn new() -> Self {
        let base = Skill::new("Nurgle's Rot", SkillCategory::Extraordinary);
        Self { base }
    }
}

impl Default for NurglesRot {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for NurglesRot {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(NurglesRot::new().get_name(), "Nurgle's Rot");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(NurglesRot::new().get_category(), SkillCategory::Extraordinary);
    }
}
