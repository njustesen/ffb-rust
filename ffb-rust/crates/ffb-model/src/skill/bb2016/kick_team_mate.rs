/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::KickTeamMate.
use crate::model::skill::Skill;
use crate::enums::SkillCategory;

pub struct KickTeamMate {
    pub base: Skill,
}

impl KickTeamMate {
    pub fn new() -> Self {
        let base = Skill::new("Kick Team-Mate", SkillCategory::Extraordinary);
        Self { base }
    }
}

impl Default for KickTeamMate {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for KickTeamMate {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(KickTeamMate::new().get_name(), "Kick Team-Mate");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(KickTeamMate::new().get_category(), SkillCategory::Extraordinary);
    }
}
