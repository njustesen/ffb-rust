/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::BugmansXxxxxx.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct BugmansXxxxxx {
    pub base: Skill,
}

impl BugmansXxxxxx {
    pub fn new() -> Self {
        let base = Skill::new("Bugman's XXXXXX", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for BugmansXxxxxx {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for BugmansXxxxxx {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(BugmansXxxxxx::new().get_name(), "Bugman's XXXXXX"); }
    #[test]
    fn category_is_correct() { assert_eq!(BugmansXxxxxx::new().get_category(), SkillCategory::Trait); }
}
