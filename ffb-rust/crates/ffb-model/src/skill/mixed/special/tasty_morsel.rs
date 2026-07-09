/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::TastyMorsel.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct TastyMorsel {
    pub base: Skill,
}

impl TastyMorsel {
    pub fn new() -> Self {
        let base = Skill::new("Tasty Morsel", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for TastyMorsel {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for TastyMorsel {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(TastyMorsel::new().get_name(), "Tasty Morsel"); }
    #[test]
    fn category_is_correct() { assert_eq!(TastyMorsel::new().get_category(), SkillCategory::Trait); }
}
