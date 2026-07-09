/// 1:1 translation of com.fumbbl.ffb.skill.common::Wrestle.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Wrestle {
    pub base: Skill,
}

impl Wrestle {
    pub fn new() -> Self {
        let base = Skill::new("Wrestle", SkillCategory::General);
        Self { base }
    }
}

impl Default for Wrestle {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Wrestle {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(Wrestle::new().get_name(), "Wrestle"); }
    #[test]
    fn category_is_correct() { assert_eq!(Wrestle::new().get_category(), SkillCategory::General); }
}
