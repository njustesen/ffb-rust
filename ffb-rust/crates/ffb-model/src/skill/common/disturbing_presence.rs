/// 1:1 translation of com.fumbbl.ffb.skill.common::DisturbingPresence.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct DisturbingPresence {
    pub base: Skill,
}

impl DisturbingPresence {
    pub fn new() -> Self {
        let base = Skill::new("Disturbing Presence", SkillCategory::Mutation);
        Self { base }
    }
}

impl Default for DisturbingPresence {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for DisturbingPresence {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(DisturbingPresence::new().get_name(), "Disturbing Presence"); }
    #[test]
    fn category_is_correct() { assert_eq!(DisturbingPresence::new().get_category(), SkillCategory::Mutation); }
}
