/// 1:1 translation of com.fumbbl.ffb.skill.mixed::StrongArm.
use crate::model::player::Player;
use crate::model::property::named_properties::NamedProperties;
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct StrongArm {
    pub base: Skill,
}

impl StrongArm {
    pub fn new() -> Self {
        let base = Skill::new("Strong Arm", SkillCategory::Strength);
        Self { base }
    }

    /// Java `canBeAssignedTo(Player<?> player)` override — Strong Arm can only be assigned to
    /// players who can already throw team-mates.
    pub fn can_be_assigned_to(&self, player: &Player) -> bool {
        player.has_skill_property(NamedProperties::CAN_THROW_TEAM_MATES)
    }
}

impl Default for StrongArm {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for StrongArm {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mirrors ffb-java StrongArmSkillTest (written against the bb2016 class);
    // the mixed edition's postConstruct registers no named properties (only a
    // PassModifier, which has no live Rust equivalent to assert against).
    // The Java `is_bb2016_edition` test is skipped (edition annotations are
    // covered elsewhere); `has_skill_properties_not_null` has no live Rust
    // equivalent (properties() returns a slice, which can never be null).

    #[test]
    fn name_is_strong_arm() { assert_eq!(StrongArm::new().get_name(), "Strong Arm"); }
    #[test]
    fn category_is_strength() { assert_eq!(StrongArm::new().get_category(), SkillCategory::Strength); }

    // Additional Rust-side logic tests beyond the Java test class.
    #[test]
    fn can_be_assigned_to_player_without_throw_team_mates_is_false() {
        let player = Player::default();
        assert!(!StrongArm::new().can_be_assigned_to(&player));
    }
    #[test]
    fn can_be_assigned_to_player_with_throw_team_mates_is_true() {
        use crate::model::skill_def::SkillWithValue;
        use crate::enums::SkillId;
        let mut player = Player::default();
        player.starting_skills.push(SkillWithValue::new(SkillId::ThrowTeamMate));
        assert!(StrongArm::new().can_be_assigned_to(&player));
    }
}
