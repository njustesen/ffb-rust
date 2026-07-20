/// 1:1 translation of com.fumbbl.ffb.skill.mixed::MyBall.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct MyBall {
    pub base: Skill,
}

impl MyBall {
    pub fn new() -> Self {
        let base = Skill::new("My Ball", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for MyBall {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for MyBall {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mirrors ffb-java MyBallSkillTest.

    #[test]
    fn name_is_my_ball() {
        assert_eq!(MyBall::new().get_name(), "My Ball");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(MyBall::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn has_prevent_regular_hand_over_action_property() {
        assert!(crate::enums::SkillId::MyBall.properties().contains(&"preventRegularHandOverAction"));
    }

    #[test]
    fn class_name_is_my_ball() {
        assert_eq!(crate::enums::SkillId::MyBall.class_name(), "MyBall");
    }
}
