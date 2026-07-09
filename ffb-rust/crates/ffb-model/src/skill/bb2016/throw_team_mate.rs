/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::ThrowTeamMate.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct ThrowTeamMate {
    pub base: Skill,
}

impl ThrowTeamMate {
    pub fn new() -> Self {
        let base = Skill::new("Throw Team-Mate", SkillCategory::Extraordinary);
        Self { base }
    }
}

impl Default for ThrowTeamMate {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for ThrowTeamMate {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(ThrowTeamMate::new().get_name(), "Throw Team-Mate");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(ThrowTeamMate::new().get_category(), SkillCategory::Extraordinary);
    }
}
