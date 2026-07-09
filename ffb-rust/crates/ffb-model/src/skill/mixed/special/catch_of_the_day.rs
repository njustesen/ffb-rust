/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::CatchOfTheDay.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct CatchOfTheDay {
    pub base: Skill,
}

impl CatchOfTheDay {
    pub fn new() -> Self {
        let base = Skill::new("Catch of the Day", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for CatchOfTheDay {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for CatchOfTheDay {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(CatchOfTheDay::new().get_name(), "Catch of the Day"); }
    #[test]
    fn category_is_correct() { assert_eq!(CatchOfTheDay::new().get_category(), SkillCategory::Trait); }
}
