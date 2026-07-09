/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::WatchOut.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct WatchOut {
    pub base: Skill,
}

impl WatchOut {
    pub fn new() -> Self {
        let base = Skill::new("Watch Out!", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for WatchOut {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for WatchOut {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(WatchOut::new().get_name(), "Watch Out!"); }
    #[test]
    fn category_is_correct() { assert_eq!(WatchOut::new().get_category(), SkillCategory::Trait); }
}
