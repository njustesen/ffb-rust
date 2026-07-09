/// 1:1 translation of com.fumbbl.ffb.skill.common::Sprint.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Sprint {
    pub base: Skill,
}

impl Sprint {
    pub fn new() -> Self {
        let base = Skill::new("Sprint", SkillCategory::Agility);
        Self { base }
    }
}

impl Default for Sprint {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Sprint {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(Sprint::new().get_name(), "Sprint"); }
    #[test]
    fn category_is_correct() { assert_eq!(Sprint::new().get_category(), SkillCategory::Agility); }
}
