/// 1:1 translation of com.fumbbl.ffb.skill.common::StripBall.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;
use crate::model::property::{NamedProperties, NamedProperty};

pub struct StripBall {
    pub base: Skill,
}

impl StripBall {
    pub fn new() -> Self {
        let mut base = Skill::new("Strip Ball", SkillCategory::General);
        // Java postConstruct(): registerProperty(NamedProperties.forceOpponentToDropBallOnPushback);
        base.register_property(Box::new(NamedProperty::new(NamedProperties::FORCE_OPPONENT_TO_DROP_BALL_ON_PUSHBACK)));
        Self { base }
    }
}

impl Default for StripBall {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for StripBall {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(StripBall::new().get_name(), "Strip Ball"); }
    #[test]
    fn category_is_correct() { assert_eq!(StripBall::new().get_category(), SkillCategory::General); }
    #[test]
    fn registers_force_opponent_to_drop_ball_on_pushback_property() {
        // Java StripBall.postConstruct() registers forceOpponentToDropBallOnPushback;
        // this would have failed before the fix since no property was registered.
        let s = StripBall::new();
        assert!(s.has_skill_property(NamedProperties::FORCE_OPPONENT_TO_DROP_BALL_ON_PUSHBACK));
    }
}
