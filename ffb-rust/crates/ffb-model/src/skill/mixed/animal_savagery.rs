/// 1:1 translation of com.fumbbl.ffb.skill.mixed::AnimalSavagery.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct AnimalSavagery {
    pub base: Skill,
}

impl AnimalSavagery {
    pub fn new() -> Self {
        let base = Skill::as_negative_trait("Animal Savagery", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for AnimalSavagery {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for AnimalSavagery {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(AnimalSavagery::new().get_name(), "Animal Savagery"); }
    #[test]
    fn category_is_correct() { assert_eq!(AnimalSavagery::new().get_category(), SkillCategory::Trait); }
    #[test]
    fn is_negative_trait() { assert!(AnimalSavagery::new().is_negative_trait()); }
}
