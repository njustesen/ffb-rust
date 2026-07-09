/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::SteadyFooting.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct SteadyFooting {
    pub base: Skill,
}

impl SteadyFooting {
    pub fn new() -> Self {
        let base = Skill::new("Steady Footing", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for SteadyFooting {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for SteadyFooting {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(SteadyFooting::new().get_name(), "Steady Footing");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(SteadyFooting::new().get_category(), SkillCategory::Trait);
    }
}
