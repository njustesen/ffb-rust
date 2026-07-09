/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::ThinkingMansTroll.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct ThinkingMansTroll {
    pub base: Skill,
}

impl ThinkingMansTroll {
    pub fn new() -> Self {
        let base = Skill::new("Thinking Man's Troll", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for ThinkingMansTroll {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for ThinkingMansTroll {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(ThinkingMansTroll::new().get_name(), "Thinking Man's Troll"); }
    #[test]
    fn category_is_correct() { assert_eq!(ThinkingMansTroll::new().get_category(), SkillCategory::Trait); }
}
