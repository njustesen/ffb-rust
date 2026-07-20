/// 1:1 translation of com.fumbbl.ffb.skill.mixed::Trickster.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Trickster {
    pub base: Skill,
}

impl Trickster {
    pub fn new() -> Self {
        let base = Skill::new("Trickster", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for Trickster {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Trickster {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mirrors ffb-java TricksterSkillTest.

    #[test]
    fn name_is_trickster() {
        assert_eq!(Trickster::new().get_name(), "Trickster");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(Trickster::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn has_can_move_before_being_blocked_property() {
        assert!(crate::enums::SkillId::Trickster.properties().contains(&"canMoveBeforeBeingBlocked"));
    }

    #[test]
    fn does_not_have_force_followup_property() {
        assert!(!crate::enums::SkillId::Trickster.properties().contains(&"forceFollowup"));
    }
}
