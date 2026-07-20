/// 1:1 translation of com.fumbbl.ffb.skill.common::Tackle.
// NOTE: Java's postConstruct() registers three `CancelSkillProperty` wrappers
// (around canRerollDodge, ignoreDefenderStumblesResult, and
// ignoresDefenderStumblesResultForFirstBlock). The Rust `CancelSkillProperty`
// (model/property/cancel_skill_property.rs) was translated with a different
// shape (wraps a `SkillId`) instead of Java's (wraps an arbitrary
// `ISkillProperty`), so there is no way to construct these registrations with
// the current infra. Deferred until CancelSkillProperty is re-ported to wrap a
// boxed ISkillProperty.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Tackle {
    pub base: Skill,
}

impl Tackle {
    pub fn new() -> Self {
        let base = Skill::new("Tackle", SkillCategory::General);
        Self { base }
    }
}

impl Default for Tackle {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Tackle {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_tackle() {
        assert_eq!(Tackle::new().get_name(), "Tackle");
    }

    #[test]
    fn category_is_general() {
        assert_eq!(Tackle::new().get_category(), SkillCategory::General);
    }

    #[test]
    fn has_can_reroll_dodge_cancel_property() {
        // Tackle must cancel the opponent's Dodge re-roll
        assert!(crate::enums::SkillId::Tackle.properties().contains(&"cancelsCanRerollDodge"));
    }

    #[test]
    fn does_not_have_force_followup_property() {
        // Tackle does not force the attacker to follow up
        assert!(!crate::enums::SkillId::Tackle.properties().contains(&"forceFollowup"));
    }
}
