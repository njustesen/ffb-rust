/// 1:1 translation of com.fumbbl.ffb.skill.common::Tentacles.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;
use crate::model::property::{NamedProperties, NamedProperty};

pub struct Tentacles {
    pub base: Skill,
}

impl Tentacles {
    pub fn new() -> Self {
        let mut base = Skill::new("Tentacles", SkillCategory::Mutation);
        // Java postConstruct(): registerProperty(NamedProperties.canHoldPlayersLeavingTacklezones);
        base.register_property(Box::new(NamedProperty::new(NamedProperties::CAN_HOLD_PLAYERS_LEAVING_TACKLEZONES)));
        Self { base }
    }
}

impl Default for Tentacles {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Tentacles {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_tentacles() {
        assert_eq!(Tentacles::new().get_name(), "Tentacles");
    }

    #[test]
    fn category_is_mutation() {
        assert_eq!(Tentacles::new().get_category(), SkillCategory::Mutation);
    }

    #[test]
    fn has_can_hold_players_leaving_tacklezones_property() {
        assert!(crate::enums::SkillId::Tentacles.properties().contains(&"canHoldPlayersLeavingTacklezones"));
    }
}
