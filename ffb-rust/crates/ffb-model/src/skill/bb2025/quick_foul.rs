/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::QuickFoul.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct QuickFoul {
    pub base: Skill,
}

impl QuickFoul {
    pub fn new() -> Self {
        let base = Skill::new("Quick Foul", SkillCategory::Devious);
        Self { base }
    }
}

impl Default for QuickFoul {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for QuickFoul {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(QuickFoul::new().get_name(), "Quick Foul");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(QuickFoul::new().get_category(), SkillCategory::Devious);
    }
}
