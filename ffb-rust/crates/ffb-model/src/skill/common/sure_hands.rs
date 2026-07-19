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
    fn name_is_correct() { assert_eq!(SureHands::new().get_name(), "Sure Hands"); }
    #[test]
    fn category_is_correct() { assert_eq!(SureHands::new().get_category(), SkillCategory::General); }
    #[test]
    fn registers_pick_up_reroll_source() {
        // Java SureHands.postConstruct() registers a reroll source for PICK_UP;
        // this would have failed before the fix since no reroll source was registered.
        let s = SureHands::new();
        let action = ReRolledAction::new("Pick Up");
        let source = s.get_reroll_source(&action);
        assert!(source.is_some());
        assert_eq!(source.unwrap().name, "Sure Hands");
    }
}
