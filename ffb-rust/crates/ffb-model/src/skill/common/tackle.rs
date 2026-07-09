/// 1:1 translation of com.fumbbl.ffb.skill.common::Tackle.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Tackle {
    pub base: Skill,
}

impl Tackle {
    pub fn new() -> Self {
        let base = Skill::new("Tackle", SkillCategory::General);
        Self { base }
    }
}

impl Default for Tackle {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Tackle {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(Tackle::new().get_name(), "Tackle"); }
    #[test]
    fn category_is_correct() { assert_eq!(Tackle::new().get_category(), SkillCategory::General); }
}
