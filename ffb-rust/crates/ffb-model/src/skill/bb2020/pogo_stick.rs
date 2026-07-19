/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::PogoStick.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct PogoStick {
    pub base: Skill,
}

impl PogoStick {
    pub fn new() -> Self {
        let base = Skill::new("Pogo Stick", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for PogoStick {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for PogoStick {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(PogoStick::new().get_name(), "Pogo Stick");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(PogoStick::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn properties_include_cancel_properties() {
        // Bug: bb2020/PogoStick.postConstruct also registers CancelSkillProperty(makesJumpingHarder)
        // and CancelSkillProperty(canAttemptToTackleJumpingPlayer), but SkillId::PogoStick.properties()
        // was missing both.
        use crate::enums::SkillId;
        let props = SkillId::PogoStick.properties();
        assert!(props.contains(&"cancelsMakesJumpingHarder"));
        assert!(props.contains(&"cancelsCanAttemptToTackleJumpingPlayer"));
    }
}
