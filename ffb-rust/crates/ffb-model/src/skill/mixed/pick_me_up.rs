/// 1:1 translation of com.fumbbl.ffb.skill.mixed::PickMeUp.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct PickMeUp {
    pub base: Skill,
}

impl PickMeUp {
    pub fn new() -> Self {
        let base = Skill::new("Pick-me-up", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for PickMeUp {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for PickMeUp {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mirrors ffb-java PickMeUpSkillTest.

    #[test]
    fn name_is_pick_me_up() {
        assert_eq!(PickMeUp::new().get_name(), "Pick-me-up");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(PickMeUp::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn has_can_stand_up_team_mates_property() {
        assert!(crate::enums::SkillId::PickMeUp.properties().contains(&"canStandUpTeamMates"));
    }

    #[test]
    fn class_name_is_pick_me_up() {
        assert_eq!(crate::enums::SkillId::PickMeUp.class_name(), "PickMeUp");
    }
}
