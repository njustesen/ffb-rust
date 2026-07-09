/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::Brawler.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Brawler {
    pub base: Skill,
}

impl Brawler {
    pub fn new() -> Self {
        let base = Skill::new("Brawler", SkillCategory::Strength);
        Self { base }
    }
}

impl Default for Brawler {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Brawler {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(Brawler::new().get_name(), "Brawler");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(Brawler::new().get_category(), SkillCategory::Strength);
    }
}
