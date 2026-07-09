/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::SavageMauling.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct SavageMauling {
    pub base: Skill,
}

impl SavageMauling {
    pub fn new() -> Self {
        let base = Skill::new("Savage Mauling", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for SavageMauling {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for SavageMauling {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(SavageMauling::new().get_name(), "Savage Mauling"); }
    #[test]
    fn category_is_correct() { assert_eq!(SavageMauling::new().get_category(), SkillCategory::Trait); }
}
