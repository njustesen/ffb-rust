/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::KickTeamMate.
use crate::model::skill::skill::Skill;
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
    use crate::enums::SkillId;

    #[test]
    fn name_is_kick_team_mate() {
        assert_eq!(KickTeamMate::new().get_name(), "Kick Team-Mate");
    }

    #[test]
    fn category_is_extraordinary() {
        assert_eq!(KickTeamMate::new().get_category(), SkillCategory::Extraordinary);
    }

    #[test]
    fn has_can_kick_team_mates_property() {
        assert!(SkillId::KickTeamMate.properties().contains(&"canKickTeamMates"));
    }
}
