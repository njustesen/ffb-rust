/// 1:1 translation of com.fumbbl.ffb.skill.common::Catch.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Catch {
    pub base: Skill,
}

impl Catch {
    pub fn new() -> Self {
        let base = Skill::new("Catch", SkillCategory::Agility);
        Self { base }
    }
}

impl Default for Catch {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Catch {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(Catch::new().get_name(), "Catch"); }
    #[test]
    fn category_is_correct() { assert_eq!(Catch::new().get_category(), SkillCategory::Agility); }
}
