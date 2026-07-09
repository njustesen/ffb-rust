/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::ReallyStupid.
use crate::model::skill::Skill;
use crate::enums::SkillCategory;

pub struct ReallyStupid {
    pub base: Skill,
}

impl ReallyStupid {
    pub fn new() -> Self {
        let base = Skill::new("Really Stupid", SkillCategory::Extraordinary);
        Self { base }
    }
}

impl Default for ReallyStupid {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for ReallyStupid {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(ReallyStupid::new().get_name(), "Really Stupid");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(ReallyStupid::new().get_category(), SkillCategory::Extraordinary);
    }
}
