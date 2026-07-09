/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::Leader.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Leader {
    pub base: Skill,
}

impl Leader {
    pub fn new() -> Self {
        let base = Skill::new("Leader", SkillCategory::General);
        Self { base }
    }
}

impl Default for Leader {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Leader {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(Leader::new().get_name(), "Leader");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(Leader::new().get_category(), SkillCategory::General);
    }
}
