/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::SavageBlow.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct SavageBlow {
    pub base: Skill,
}

impl SavageBlow {
    pub fn new() -> Self {
        let base = Skill::new("Savage Blow", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for SavageBlow {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for SavageBlow {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(SavageBlow::new().get_name(), "Savage Blow"); }
    #[test]
    fn category_is_correct() { assert_eq!(SavageBlow::new().get_category(), SkillCategory::Trait); }
}
