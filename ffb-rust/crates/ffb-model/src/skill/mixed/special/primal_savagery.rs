/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::PrimalSavagery.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct PrimalSavagery {
    pub base: Skill,
}

impl PrimalSavagery {
    pub fn new() -> Self {
        let base = Skill::new("Primal Savagery", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for PrimalSavagery {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for PrimalSavagery {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(PrimalSavagery::new().get_name(), "Primal Savagery"); }
    #[test]
    fn category_is_correct() { assert_eq!(PrimalSavagery::new().get_category(), SkillCategory::Trait); }
}
