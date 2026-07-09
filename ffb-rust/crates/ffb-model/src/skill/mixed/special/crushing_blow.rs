/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::CrushingBlow.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct CrushingBlow {
    pub base: Skill,
}

impl CrushingBlow {
    pub fn new() -> Self {
        let base = Skill::new("Crushing Blow", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for CrushingBlow {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for CrushingBlow {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(CrushingBlow::new().get_name(), "Crushing Blow"); }
    #[test]
    fn category_is_correct() { assert_eq!(CrushingBlow::new().get_category(), SkillCategory::Trait); }
}
