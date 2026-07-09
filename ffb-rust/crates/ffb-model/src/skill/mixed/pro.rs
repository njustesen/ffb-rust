/// 1:1 translation of com.fumbbl.ffb.skill.mixed::Pro.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Pro {
    pub base: Skill,
}

impl Pro {
    pub fn new() -> Self {
        let base = Skill::new("Pro", SkillCategory::General);
        Self { base }
    }
}

impl Default for Pro {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Pro {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(Pro::new().get_name(), "Pro"); }
    #[test]
    fn category_is_correct() { assert_eq!(Pro::new().get_category(), SkillCategory::General); }
}
