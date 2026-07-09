/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::GoredByTheBull.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct GoredByTheBull {
    pub base: Skill,
}

impl GoredByTheBull {
    pub fn new() -> Self {
        let base = Skill::new("Gored By The Bull", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for GoredByTheBull {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for GoredByTheBull {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(GoredByTheBull::new().get_name(), "Gored By The Bull"); }
    #[test]
    fn category_is_correct() { assert_eq!(GoredByTheBull::new().get_category(), SkillCategory::Trait); }
}
