/// 1:1 translation of com.fumbbl.ffb.skill.mixed::Cannoneer.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Cannoneer {
    pub base: Skill,
}

impl Cannoneer {
    pub fn new() -> Self {
        let base = Skill::new("Cannoneer", SkillCategory::Passing);
        Self { base }
    }
}

impl Default for Cannoneer {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Cannoneer {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(Cannoneer::new().get_name(), "Cannoneer"); }
    #[test]
    fn category_is_correct() { assert_eq!(Cannoneer::new().get_category(), SkillCategory::Passing); }
}
