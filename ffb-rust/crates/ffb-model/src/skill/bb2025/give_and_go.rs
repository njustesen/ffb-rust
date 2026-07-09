/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::GiveAndGo.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct GiveAndGo {
    pub base: Skill,
}

impl GiveAndGo {
    pub fn new() -> Self {
        let base = Skill::new("Give and Go", SkillCategory::Passing);
        Self { base }
    }
}

impl Default for GiveAndGo {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for GiveAndGo {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(GiveAndGo::new().get_name(), "Give and Go");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(GiveAndGo::new().get_category(), SkillCategory::Passing);
    }
}
