/// 1:1 translation of com.fumbbl.ffb.skill.mixed::Claws.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Claws {
    pub base: Skill,
}

impl Claws {
    pub fn new() -> Self {
        let base = Skill::new("Claws", SkillCategory::Mutation);
        Self { base }
    }
}

impl Default for Claws {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Claws {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(Claws::new().get_name(), "Claws"); }
    #[test]
    fn category_is_correct() { assert_eq!(Claws::new().get_category(), SkillCategory::Mutation); }
}
