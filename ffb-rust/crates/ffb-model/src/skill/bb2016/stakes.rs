/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::Stakes.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Stakes {
    pub base: Skill,
}

impl Stakes {
    pub fn new() -> Self {
        let base = Skill::new("Stakes", SkillCategory::Extraordinary);
        Self { base }
    }
}

impl Default for Stakes {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Stakes {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(Stakes::new().get_name(), "Stakes");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(Stakes::new().get_category(), SkillCategory::Extraordinary);
    }
}
