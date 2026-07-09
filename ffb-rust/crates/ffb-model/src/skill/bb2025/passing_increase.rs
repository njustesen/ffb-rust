/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::PassingIncrease.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct PassingIncrease {
    pub base: Skill,
}

impl PassingIncrease {
    pub fn new() -> Self {
        let base = Skill::new("+PA", SkillCategory::StatIncrease);
        Self { base }
    }
}

impl Default for PassingIncrease {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for PassingIncrease {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(PassingIncrease::new().get_name(), "+PA");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(PassingIncrease::new().get_category(), SkillCategory::StatIncrease);
    }
}
