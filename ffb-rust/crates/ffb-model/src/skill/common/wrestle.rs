/// 1:1 translation of com.fumbbl.ffb.skill.common::Wrestle.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;
use crate::model::property::{NamedProperties, NamedProperty};

pub struct Wrestle {
    pub base: Skill,
}

impl Wrestle {
    pub fn new() -> Self {
        let mut base = Skill::new("Wrestle", SkillCategory::General);
        // Java postConstruct(): registerProperty(NamedProperties.canTakeDownPlayersWithHimOnBothDown);
        base.register_property(Box::new(NamedProperty::new(NamedProperties::CAN_TAKE_DOWN_PLAYERS_WITH_HIM_ON_BOTH_DOWN)));
        Self { base }
    }

    /// Java `getSkillUseDescription()` override.
    pub fn get_skill_use_description(&self) -> Option<Vec<String>> {
        Some(vec![
            "Using Wrestle will put down both you and your opponent.".to_string(),
            "No Armor Roll is made. The ball carrier drops the ball.".to_string(),
        ])
    }
}

impl Default for Wrestle {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Wrestle {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(Wrestle::new().get_name(), "Wrestle"); }
    #[test]
    fn category_is_correct() { assert_eq!(Wrestle::new().get_category(), SkillCategory::General); }
    #[test]
    fn registers_can_take_down_players_with_him_on_both_down_property() {
        // Java Wrestle.postConstruct() registers canTakeDownPlayersWithHimOnBothDown;
        // this would have failed before the fix since no property was registered.
        let w = Wrestle::new();
        assert!(w.has_skill_property(NamedProperties::CAN_TAKE_DOWN_PLAYERS_WITH_HIM_ON_BOTH_DOWN));
    }
    #[test]
    fn get_skill_use_description_matches_java() {
        // Java Wrestle.getSkillUseDescription() overrides the base (null) and
        // returns a two-line description; the base impl always returned None.
        let w = Wrestle::new();
        let desc = w.get_skill_use_description().expect("Wrestle must override description");
        assert_eq!(desc.len(), 2);
        assert_eq!(desc[0], "Using Wrestle will put down both you and your opponent.");
        assert_eq!(desc[1], "No Armor Roll is made. The ball carrier drops the ball.");
    }
}
