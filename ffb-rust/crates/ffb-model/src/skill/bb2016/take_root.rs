/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::TakeRoot.
use crate::model::skill::Skill;
use crate::enums::SkillCategory;

pub struct TakeRoot {
    pub base: Skill,
}

impl TakeRoot {
    pub fn new() -> Self {
        let base = Skill::new("Take Root", SkillCategory::Extraordinary);
        Self { base }
    }
}

impl Default for TakeRoot {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for TakeRoot {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(TakeRoot::new().get_name(), "Take Root");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(TakeRoot::new().get_category(), SkillCategory::Extraordinary);
    }
}
