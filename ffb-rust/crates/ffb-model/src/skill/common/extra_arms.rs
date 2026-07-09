/// 1:1 translation of com.fumbbl.ffb.skill.common::ExtraArms.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct ExtraArms {
    pub base: Skill,
}

impl ExtraArms {
    pub fn new() -> Self {
        let base = Skill::new("Extra Arms", SkillCategory::Mutation);
        Self { base }
    }
}

impl Default for ExtraArms {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for ExtraArms {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(ExtraArms::new().get_name(), "Extra Arms"); }
    #[test]
    fn category_is_correct() { assert_eq!(ExtraArms::new().get_category(), SkillCategory::Mutation); }
}
