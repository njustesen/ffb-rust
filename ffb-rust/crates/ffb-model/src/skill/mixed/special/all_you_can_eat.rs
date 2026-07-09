/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::AllYouCanEat.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct AllYouCanEat {
    pub base: Skill,
}

impl AllYouCanEat {
    pub fn new() -> Self {
        let base = Skill::new("All You Can Eat", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for AllYouCanEat {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for AllYouCanEat {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(AllYouCanEat::new().get_name(), "All You Can Eat"); }
    #[test]
    fn category_is_correct() { assert_eq!(AllYouCanEat::new().get_category(), SkillCategory::Trait); }
}
