/// 1:1 translation of com.fumbbl.ffb.skill.common::Dauntless.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Dauntless {
    pub base: Skill,
}

impl Dauntless {
    pub fn new() -> Self {
        let base = Skill::new("Dauntless", SkillCategory::General);
        Self { base }
    }
}

impl Default for Dauntless {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Dauntless {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(Dauntless::new().get_name(), "Dauntless"); }
    #[test]
    fn category_is_correct() { assert_eq!(Dauntless::new().get_category(), SkillCategory::General); }
}
