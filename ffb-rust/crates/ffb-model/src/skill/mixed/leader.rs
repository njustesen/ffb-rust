/// 1:1 translation of com.fumbbl.ffb.skill.mixed::Leader.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Leader {
    pub base: Skill,
}

impl Leader {
    pub fn new() -> Self {
        let base = Skill::new("Leader", SkillCategory::Passing);
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

    // Mirrors ffb-java LeaderSkillTest.

    #[test]
    fn name_is_leader() {
        assert_eq!(Leader::new().get_name(), "Leader");
    }

    #[test]
    fn category_is_passing() {
        assert_eq!(Leader::new().get_category(), SkillCategory::Passing);
    }

    #[test]
    fn has_grants_team_reroll_when_on_pitch_property() {
        assert!(crate::enums::SkillId::Leader.properties().contains(&"grantsTeamReRollWhenOnPitch"));
    }

    #[test]
    fn class_name_is_leader() {
        assert_eq!(crate::enums::SkillId::Leader.class_name(), "Leader");
    }
}
