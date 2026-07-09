/// 1:1 translation of com.fumbbl.ffb.skill.mixed::Loner.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Loner {
    pub base: Skill,
}

impl Loner {
    pub fn new() -> Self {
        let base = Skill::new("Loner", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for Loner {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Loner {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(Loner::new().get_name(), "Loner"); }
    #[test]
    fn category_is_correct() { assert_eq!(Loner::new().get_category(), SkillCategory::Trait); }
}
