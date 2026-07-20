/// 1:1 translation of com.fumbbl.ffb.skill.common::SureHands.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, ReRollSource};
use crate::model::re_rolled_action::ReRolledAction;

// NOTE: Java's postConstruct() also does:
//   registerProperty(new CancelSkillProperty(NamedProperties.forceOpponentToDropBallOnPushback));
// The Rust `CancelSkillProperty` (model/property/cancel_skill_property.rs) was translated with a
// different shape (wraps a `SkillId`) instead of Java's (wraps an arbitrary `ISkillProperty`), so
// there is no way to construct "cancel the forceOpponentToDropBallOnPushback property" with the
// current infra. Deferred until CancelSkillProperty is re-ported to wrap a boxed ISkillProperty.

pub struct SureHands {
    pub base: Skill,
}

impl SureHands {
    pub fn new() -> Self {
        let mut base = Skill::new("Sure Hands", SkillCategory::General);
        // Java postConstruct(): registerRerollSource(ReRolledActions.PICK_UP, ReRollSources.SURE_HANDS);
        base.register_reroll_source(
            ReRolledAction::new("Pick Up"),
            ReRollSource::new("Sure Hands"),
        );
        Self { base }
    }
}

impl Default for SureHands {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for SureHands {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_sure_hands() {
        assert_eq!(SureHands::new().get_name(), "Sure Hands");
    }

    #[test]
    fn category_is_general() {
        assert_eq!(SureHands::new().get_category(), SkillCategory::General);
    }

    #[test]
    fn has_cancel_strip_ball_property() {
        // Java: assertTrue(skill.hasSkillProperty(new CancelSkillProperty(
        // NamedProperties.forceOpponentToDropBallOnPushback)))
        assert!(crate::enums::SkillId::SureHands.properties().contains(&"cancelsForceOpponentToDropBallOnPushback"));
    }

    #[test]
    fn does_not_have_force_followup_property() {
        // Sure Hands does not force follow-up
        assert!(!crate::enums::SkillId::SureHands.properties().contains(&"forceFollowup"));
    }

    #[test]
    fn has_sure_hands_reroll_source() {
        // Java: assertNotNull(skill.getRerollSource(ReRolledActions.PICK_UP)) —
        // mirrored against the live SkillId::reroll_sources() table (the
        // per-struct register_reroll_source map is inert; Java PICK_UP is
        // "PICKUP" in the Rust engine's action vocabulary).
        assert!(crate::enums::SkillId::SureHands.reroll_sources().iter().any(|(a, _)| *a == "PICKUP"));
    }
}
