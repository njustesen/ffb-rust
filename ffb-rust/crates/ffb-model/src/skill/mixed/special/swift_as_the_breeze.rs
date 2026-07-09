/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::SwiftAsTheBreeze.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct SwiftAsTheBreeze {
    pub base: Skill,
}

impl SwiftAsTheBreeze {
    pub fn new() -> Self {
        let base = Skill::new("Swift As The Breeze", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for SwiftAsTheBreeze {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for SwiftAsTheBreeze {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(SwiftAsTheBreeze::new().get_name(), "Swift As The Breeze"); }
    #[test]
    fn category_is_correct() { assert_eq!(SwiftAsTheBreeze::new().get_category(), SkillCategory::Trait); }
}
