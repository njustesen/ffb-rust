/// 1:1 translation of com.fumbbl.ffb.skill.mixed::Drunkard.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Drunkard {
    pub base: Skill,
}

impl Drunkard {
    pub fn new() -> Self {
        let base = Skill::new("Drunkard", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for Drunkard {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Drunkard {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(Drunkard::new().get_name(), "Drunkard"); }
    #[test]
    fn category_is_correct() { assert_eq!(Drunkard::new().get_category(), SkillCategory::Trait); }
}
