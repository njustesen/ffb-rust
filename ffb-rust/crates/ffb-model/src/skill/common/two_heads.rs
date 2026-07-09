/// 1:1 translation of com.fumbbl.ffb.skill.common::TwoHeads.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct TwoHeads {
    pub base: Skill,
}

impl TwoHeads {
    pub fn new() -> Self {
        let base = Skill::new("Two Heads", SkillCategory::Mutation);
        Self { base }
    }
}

impl Default for TwoHeads {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for TwoHeads {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(TwoHeads::new().get_name(), "Two Heads"); }
    #[test]
    fn category_is_correct() { assert_eq!(TwoHeads::new().get_category(), SkillCategory::Mutation); }
}
