/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::LoneFouler.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct LoneFouler {
    pub base: Skill,
}

impl LoneFouler {
    pub fn new() -> Self {
        let base = Skill::new("Lone Fouler", SkillCategory::Devious);
        Self { base }
    }
}

impl Default for LoneFouler {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for LoneFouler {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(LoneFouler::new().get_name(), "Lone Fouler");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(LoneFouler::new().get_category(), SkillCategory::Devious);
    }
}
