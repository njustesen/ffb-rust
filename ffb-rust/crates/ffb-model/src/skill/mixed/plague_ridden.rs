/// 1:1 translation of com.fumbbl.ffb.skill.mixed::PlagueRidden.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct PlagueRidden {
    pub base: Skill,
}

impl PlagueRidden {
    pub fn new() -> Self {
        let base = Skill::new("Plague Ridden", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for PlagueRidden {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for PlagueRidden {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(PlagueRidden::new().get_name(), "Plague Ridden"); }
    #[test]
    fn category_is_correct() { assert_eq!(PlagueRidden::new().get_category(), SkillCategory::Trait); }
}
