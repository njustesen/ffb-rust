/// 1:1 translation of com.fumbbl.ffb.skill.mixed::Dodge.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Dodge {
    pub base: Skill,
}

impl Dodge {
    pub fn new() -> Self {
        let base = Skill::new("Dodge", SkillCategory::Agility);
        Self { base }
    }
}

impl Default for Dodge {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Dodge {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(Dodge::new().get_name(), "Dodge"); }
    #[test]
    fn category_is_correct() { assert_eq!(Dodge::new().get_category(), SkillCategory::Agility); }
}
